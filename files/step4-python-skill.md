# Step 4: Python Skill Wrapper

The skill wrapper provides the ForgeAgents-facing API.

## Key Constraint

**This skill NEVER writes files directly.**

All data comes as base64 from the bridge. Agents can process it in memory but cannot persist to disk through this interface.

## Directory Structure

```
forgeagents-forgeimages/
└── skill/
    ├── __init__.py
    └── forgeimages_skill.py
```

## skill/forgeimages_skill.py

```python
"""
ForgeImages Skill for ForgeAgents.

CRITICAL CONSTRAINTS:
- This skill NEVER writes files directly
- This skill ONLY calls the bridge service
- All validation is enforced by the bridge/Rust
"""

from typing import Optional
import httpx

from bridge.models import (
    AssetInput,
    CompileRequest,
    CompileResponse,
    ValidationResult,
    TemplateInfo,
)


class ForgeImagesSkill:
    """
    ForgeImages skill for ForgeAgents.
    
    Agents CAN:
    - List available templates
    - Validate assets before compilation
    - Request compilation (validation enforced)
    
    Agents CANNOT:
    - Bypass validation
    - Write files directly
    - Override template constraints
    """
    
    def __init__(self, bridge_url: str = "http://localhost:8789"):
        self.bridge_url = bridge_url.rstrip("/")
        self.client = httpx.Client(timeout=30.0)
    
    def list_templates(self) -> list[TemplateInfo]:
        """List all available templates."""
        response = self.client.get(f"{self.bridge_url}/templates")
        response.raise_for_status()
        return [TemplateInfo(**t) for t in response.json()]
    
    def validate_asset(
        self,
        template_id: str,
        width: int,
        height: int,
        color_count: Optional[int] = None,
    ) -> ValidationResult:
        """
        Validate an asset against a template.
        
        Returns ValidationResult with violations if invalid.
        """
        asset_input = AssetInput(
            width=width,
            height=height,
            color_count=color_count,
        )
        
        response = self.client.post(
            f"{self.bridge_url}/validate",
            params={"template_id": template_id},
            json=asset_input.model_dump(exclude_none=True),
        )
        response.raise_for_status()
        return ValidationResult(**response.json())
    
    def compile_asset(
        self,
        template_id: str,
        width: int,
        height: int,
        color_count: Optional[int] = None,
        source_data: Optional[str] = None,
        seed: Optional[int] = None,
        prompt: Optional[str] = None,
    ) -> CompileResponse:
        """
        Compile an asset.
        
        Raises httpx.HTTPStatusError with status 422 if validation fails.
        
        NOTE: This method does NOT bypass validation.
        """
        request = CompileRequest(
            template_id=template_id,
            asset_input=AssetInput(
                width=width,
                height=height,
                color_count=color_count,
            ),
            source_data=source_data,
            seed=seed,
            prompt=prompt,
        )
        
        response = self.client.post(
            f"{self.bridge_url}/compile",
            json=request.model_dump(exclude_none=True),
        )
        
        # Return error details for 422
        if response.status_code == 422:
            return CompileResponse(
                success=False,
                error=response.json().get("detail", {}).get("message", "Validation failed"),
            )
        
        response.raise_for_status()
        return CompileResponse(**response.json())
    
    def close(self):
        """Close the HTTP client."""
        self.client.close()
    
    def __enter__(self):
        return self
    
    def __exit__(self, *args):
        self.close()
```

## skill/__init__.py

```python
"""ForgeImages Skill for ForgeAgents."""
from .forgeimages_skill import ForgeImagesSkill

__all__ = ["ForgeImagesSkill"]
```

## Usage Example

```python
from skill import ForgeImagesSkill

with ForgeImagesSkill() as skill:
    # List templates
    templates = skill.list_templates()
    print(f"Available templates: {[t.id for t in templates]}")
    
    # Validate before compile
    result = skill.validate_asset("pwa-icon", 1024, 1024)
    if not result.valid:
        print(f"Validation failed: {result.violations}")
    
    # Compile
    response = skill.compile_asset("pwa-icon", 1024, 1024)
    if response.success:
        # Access base64 data (NOT file paths)
        for export in response.asset.exports:
            print(f"Export: {export.filename}, hash: {export.hash}")
            # export.data_base64 contains the actual data
    else:
        print(f"Compilation failed: {response.error}")
```

## What's NOT Here

The skill intentionally excludes:

- `write_file()` - No file writing
- `save_to_disk()` - No disk access
- `export_path` - No file paths
- `override_validation` - No bypass

These omissions are the enforcement mechanism.
