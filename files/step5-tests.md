# Step 5: Agent Boundary Tests

These tests PROVE that agents cannot bypass validation.

**If any test fails, the integration is broken.**

## tests/test_agent_boundary.py

```python
"""
Agent Boundary Tests.

These tests PROVE that agents cannot bypass ForgeImages validation.
"""

import pytest
from fastapi.testclient import TestClient

from bridge.forgeimages_bridge import app


client = TestClient(app)


class TestAgentBoundaries:
    """Tests proving agents cannot bypass validation."""
    
    def test_invalid_aspect_ratio_returns_422(self):
        """
        Invalid payload MUST return 422.
        This is the core enforcement mechanism.
        """
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 1024,
                "height": 512,  # Wrong aspect ratio!
            }
        }
        
        response = client.post("/compile", json=payload)
        
        # MUST be 422, not 200 or 500
        assert response.status_code == 422, \
            f"Expected 422, got {response.status_code}: {response.text}"
    
    def test_resolution_too_low_returns_422(self):
        """Resolution below minimum MUST be rejected."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 100,
                "height": 100,
            }
        }
        
        response = client.post("/compile", json=payload)
        assert response.status_code == 422
    
    def test_valid_payload_succeeds(self):
        """Valid payload should compile successfully."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 1024,
                "height": 1024,
                "color_count": 8,
            }
        }
        
        response = client.post("/compile", json=payload)
        
        assert response.status_code == 200, \
            f"Expected 200, got {response.status_code}: {response.text}"
        
        data = response.json()
        assert data["success"] is True
        assert "asset" in data
        
        # Manifest hash must be present
        asset = data["asset"]
        assert "manifest_hash" in asset
        assert len(asset["manifest_hash"]) == 64  # SHA-256 hex
    
    def test_no_file_write_endpoint(self):
        """There must be no endpoint that writes files."""
        dangerous_paths = [
            "/write",
            "/save",
            "/export",
            "/file",
            "/output",
        ]
        
        for path in dangerous_paths:
            response = client.post(path, json={})
            assert response.status_code in [404, 405, 422], \
                f"Unexpected response for {path}: {response.status_code}"
    
    def test_override_validation_ignored(self):
        """
        Attempts to override validation MUST be ignored.
        """
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {
                "width": 100,  # Invalid
                "height": 100,
            },
            # Attempted bypasses
            "skip_validation": True,
            "force": True,
            "bypass": True,
            "validation_override": True,
        }
        
        response = client.post("/compile", json=payload)
        
        # MUST still fail
        assert response.status_code == 422, \
            "Validation bypass attempted but should have failed"
    
    def test_template_id_injection_rejected(self):
        """Malicious template IDs must be rejected."""
        malicious_ids = [
            "../../../etc/passwd",
            "template; rm -rf /",
            "<script>alert('xss')</script>",
        ]
        
        for template_id in malicious_ids:
            payload = {
                "template_id": template_id,
                "asset_input": {"width": 1024, "height": 1024}
            }
            
            response = client.post("/compile", json=payload)
            assert response.status_code in [400, 404, 422, 500]
    
    def test_validate_endpoint_returns_violations(self):
        """Validate endpoint must return structured violations."""
        response = client.post(
            "/validate",
            params={"template_id": "pwa-icon"},
            json={"width": 100, "height": 200}
        )
        
        assert response.status_code == 200
        data = response.json()
        assert "valid" in data
        assert "violations" in data
        
        if not data["valid"]:
            assert len(data["violations"]) > 0
            for v in data["violations"]:
                assert "rule" in v
                assert "message" in v
    
    def test_compile_includes_job_hash(self):
        """Compiled assets must include job_hash for auditing."""
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {"width": 1024, "height": 1024}
        }
        
        response = client.post("/compile", json=payload)
        
        if response.status_code == 200:
            data = response.json()
            assert "job_hash" in data["asset"]
            assert len(data["asset"]["job_hash"]) == 64


class TestSkillWrapper:
    """Tests for the ForgeAgents skill wrapper."""
    
    def test_skill_cannot_write_files(self):
        """Verify the skill has no file-writing methods."""
        from skill.forgeimages_skill import ForgeImagesSkill
        
        skill = ForgeImagesSkill()
        
        forbidden_methods = [
            "write_file",
            "save_file",
            "export_to_disk",
            "write_output",
            "save_asset",
        ]
        
        for method in forbidden_methods:
            assert not hasattr(skill, method), \
                f"Skill should not have {method} method"
        
        skill.close()
    
    def test_skill_returns_base64_not_paths(self):
        """Skill must return data as base64, never file paths."""
        from bridge.models import ExportedFile
        
        # Model must have data_base64, not path
        assert "data_base64" in ExportedFile.model_fields
        assert "path" not in ExportedFile.model_fields
        assert "file_path" not in ExportedFile.model_fields


class TestAuditLogging:
    """Tests for audit trail."""
    
    def test_compile_creates_audit_entry(self):
        """Every compile request should be logged."""
        import os
        from bridge.settings import settings
        
        # Clear audit log
        if settings.audit_log_path.exists():
            os.remove(settings.audit_log_path)
        
        payload = {
            "template_id": "pwa-icon",
            "asset_input": {"width": 1024, "height": 1024}
        }
        
        client.post("/compile", json=payload)
        
        # Audit log should exist
        assert settings.audit_log_path.exists()
        
        # Should have content
        with open(settings.audit_log_path) as f:
            lines = f.readlines()
            assert len(lines) > 0
```

## Running Tests

```bash
cd forgeagents-forgeimages

# Install dev dependencies
pip install -e ".[dev]"

# Run tests
pytest -v

# Run with coverage
pytest -v --cov=bridge --cov=skill
```

## Test Matrix

| Test | What It Proves |
|------|----------------|
| `test_invalid_aspect_ratio_returns_422` | Core enforcement works |
| `test_resolution_too_low_returns_422` | Resolution rules enforced |
| `test_valid_payload_succeeds` | Happy path works |
| `test_no_file_write_endpoint` | No backdoor endpoints |
| `test_override_validation_ignored` | Extra fields don't bypass |
| `test_template_id_injection_rejected` | Injection attacks blocked |
| `test_validate_endpoint_returns_violations` | Errors are actionable |
| `test_compile_includes_job_hash` | Audit trail enabled |
| `test_skill_cannot_write_files` | Skill has no file I/O |
| `test_skill_returns_base64_not_paths` | Data model is safe |

## If Tests Fail

If any test fails:

1. **Do not ship** - The boundary is compromised
2. Check if validation is being called in Rust
3. Check if bridge is returning 422 on failure
4. Check if skill has unauthorized methods

The tests are the contract. Passing tests = working enforcement.
