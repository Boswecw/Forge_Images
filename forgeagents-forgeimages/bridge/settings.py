"""
Bridge configuration settings.
"""

from pathlib import Path
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Bridge configuration."""

    # Rust CLI path
    cli_path: Path = Path("../forgeimages-core/target/release/forgeimages-cli")

    # Templates directory
    templates_dir: Path = Path("../forgeimages-core/templates")

    # Audit log path
    audit_log_path: Path = Path("./audit.jsonl")

    # Request limits
    max_request_size_mb: int = 10
    max_payload_size_kb: int = 512

    # Engine version (for job hash when CLI unavailable)
    engine_version: str = "1.0.0"

    class Config:
        env_prefix = "FORGEIMAGES_"


settings = Settings()
