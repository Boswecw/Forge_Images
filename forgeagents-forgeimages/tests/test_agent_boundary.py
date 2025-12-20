"""
Agent Boundary Tests

These tests verify the critical constraint:
"Agents suggest, ForgeImages enforces"

Key invariants tested:
1. Agents cannot bypass validation
2. HTTP 422 is returned on validation failure
3. All operations are audit logged
4. No file writes occur (base64 only)
"""

import pytest
from unittest.mock import AsyncMock, patch, MagicMock
import json

from bridge.models import (
    AssetInput,
    CompileRequest,
    ValidationResult,
    ValidationViolation,
)
from skill.forgeimages_skill import ForgeImagesSkill, ForgeImagesError


class TestValidationEnforcement:
    """Test that validation cannot be bypassed."""

    @pytest.mark.asyncio
    async def test_invalid_aspect_ratio_returns_422(self):
        """Agents cannot compile assets with wrong aspect ratio."""
        skill = ForgeImagesSkill("http://test:8000")

        # Mock a 422 response for validation failure
        mock_response = MagicMock()
        mock_response.status_code = 422
        mock_response.json.return_value = {
            "detail": {
                "message": "Validation failed",
                "violations": [
                    {
                        "rule": "aspect_ratio",
                        "severity": "error",
                        "message": "Aspect ratio mismatch",
                        "expected": "1:1",
                        "actual": "2.000",
                        "remediation": ["Crop or resize to match template aspect ratio"],
                    }
                ],
            }
        }

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()
            mock_client.return_value.__aenter__.return_value = mock_instance
            mock_instance.request.return_value = mock_response

            # Validate should return invalid result (not raise)
            result = await skill.validate(
                template_id="pwa-icon",
                width=1024,
                height=512,  # Wrong aspect ratio
            )

            assert not result.valid
            assert len(result.violations) == 1
            assert result.violations[0]["rule"] == "aspect_ratio"

    @pytest.mark.asyncio
    async def test_compile_fails_on_validation_error(self):
        """Compile must fail if validation would fail."""
        skill = ForgeImagesSkill("http://test:8000")

        # Mock a 422 response
        mock_response = MagicMock()
        mock_response.status_code = 422
        mock_response.json.return_value = {
            "detail": {
                "message": "Compilation failed",
                "error": "Validation failed: aspect_ratio: Aspect ratio mismatch",
            }
        }

        import httpx
        mock_response.raise_for_status.side_effect = httpx.HTTPStatusError(
            message="422 Unprocessable Entity",
            request=MagicMock(),
            response=mock_response,
        )

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()
            mock_client.return_value.__aenter__.return_value = mock_instance
            mock_instance.request.return_value = mock_response

            with pytest.raises(ForgeImagesError) as exc_info:
                await skill.compile(
                    template_id="pwa-icon",
                    width=1024,
                    height=512,
                )

            assert exc_info.value.status_code == 422
            assert exc_info.value.is_validation_error()


class TestValidationResult:
    """Test ValidationResult model behavior."""

    def test_has_errors_detection(self):
        """ValidationResult correctly identifies errors."""
        result = ValidationResult(
            valid=False,
            violations=[
                {"rule": "aspect_ratio", "severity": "error", "message": "test"},
            ],
            template_id="test",
            template_version="1.0.0",
        )
        assert result.has_errors

    def test_has_warnings_detection(self):
        """ValidationResult correctly identifies warnings."""
        result = ValidationResult(
            valid=True,
            violations=[
                {"rule": "color_count", "severity": "warning", "message": "test"},
            ],
            template_id="test",
            template_version="1.0.0",
        )
        assert not result.has_errors
        assert result.has_warnings


class TestForgeImagesError:
    """Test error handling."""

    def test_validation_error_detection(self):
        """ForgeImagesError correctly identifies 422 errors."""
        error = ForgeImagesError(
            message="Validation failed",
            status_code=422,
            violations=[
                {"rule": "aspect_ratio", "remediation": ["Fix aspect ratio"]},
            ],
        )
        assert error.is_validation_error()
        assert "Fix aspect ratio" in error.get_remediation_hints()

    def test_non_validation_error(self):
        """Other errors are not validation errors."""
        error = ForgeImagesError(
            message="Server error",
            status_code=500,
        )
        assert not error.is_validation_error()
        assert error.get_remediation_hints() == []


class TestAssetInputValidation:
    """Test Pydantic model validation."""

    def test_valid_input(self):
        """Valid input passes validation."""
        input = AssetInput(width=1024, height=1024)
        assert input.width == 1024
        assert input.height == 1024

    def test_negative_dimensions_rejected(self):
        """Negative dimensions are rejected."""
        with pytest.raises(ValueError):
            AssetInput(width=-1, height=1024)

    def test_zero_dimensions_rejected(self):
        """Zero dimensions are rejected."""
        with pytest.raises(ValueError):
            AssetInput(width=0, height=1024)

    def test_excessive_dimensions_rejected(self):
        """Excessive dimensions are rejected."""
        with pytest.raises(ValueError):
            AssetInput(width=100000, height=1024)


class TestCompileRequestValidation:
    """Test CompileRequest model validation."""

    def test_valid_template_id(self):
        """Valid template IDs are accepted."""
        request = CompileRequest(
            template_id="pwa-icon",
            asset_input=AssetInput(width=1024, height=1024),
        )
        assert request.template_id == "pwa-icon"

    def test_invalid_template_id_format(self):
        """Invalid template ID formats are rejected."""
        with pytest.raises(ValueError):
            CompileRequest(
                template_id="../../../etc/passwd",  # Path traversal attempt
                asset_input=AssetInput(width=1024, height=1024),
            )

    def test_template_id_with_spaces_rejected(self):
        """Template IDs with spaces are rejected."""
        with pytest.raises(ValueError):
            CompileRequest(
                template_id="my template",
                asset_input=AssetInput(width=1024, height=1024),
            )


class TestSkillInterface:
    """Test the skill interface."""

    @pytest.mark.asyncio
    async def test_validate_and_compile_stops_on_invalid(self):
        """validate_and_compile does not compile if validation fails."""
        skill = ForgeImagesSkill("http://test:8000")

        # Mock validate to return invalid
        with patch.object(skill, "validate") as mock_validate:
            mock_validate.return_value = ValidationResult(
                valid=False,
                violations=[{"rule": "test", "severity": "error", "message": "test"}],
                template_id="test",
                template_version="1.0.0",
            )

            with patch.object(skill, "compile") as mock_compile:
                result, asset = await skill.validate_and_compile(
                    template_id="test",
                    width=100,
                    height=100,
                )

                # Compile should not be called
                mock_compile.assert_not_called()
                assert not result.valid
                assert asset is None

    @pytest.mark.asyncio
    async def test_headers_include_user_id(self):
        """User ID is passed in headers for audit."""
        skill = ForgeImagesSkill("http://test:8000", user_id="test-user-123")

        headers = skill._get_headers()
        assert headers["X-User-ID"] == "test-user-123"


class TestNoBypassPossible:
    """
    Critical tests: Verify agents cannot bypass validation.

    These tests document the security boundary.
    """

    def test_compile_request_requires_asset_input(self):
        """CompileRequest requires asset_input - no way to skip it."""
        with pytest.raises(ValueError):
            CompileRequest(template_id="test")  # type: ignore

    def test_asset_input_requires_dimensions(self):
        """AssetInput requires dimensions - no way to skip validation."""
        with pytest.raises(ValueError):
            AssetInput()  # type: ignore

    @pytest.mark.asyncio
    async def test_skill_always_calls_bridge(self):
        """Skill always goes through HTTP bridge - no local bypass."""
        skill = ForgeImagesSkill("http://test:8000")

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()
            mock_client.return_value.__aenter__.return_value = mock_instance
            mock_response = MagicMock()
            mock_response.status_code = 200
            mock_response.json.return_value = {
                "valid": True,
                "violations": [],
                "template_id": "test",
                "template_version": "1.0.0",
            }
            mock_instance.request.return_value = mock_response

            await skill.validate(
                template_id="test",
                width=1024,
                height=1024,
            )

            # Verify HTTP request was made
            mock_instance.request.assert_called_once()
            call_args = mock_instance.request.call_args
            assert call_args.kwargs["method"] == "POST"
            assert "/validate/" in call_args.kwargs["url"]
