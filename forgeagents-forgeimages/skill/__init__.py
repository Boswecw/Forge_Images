"""
ForgeImages Skill - Agent-facing interface for image operations.

This module provides the skill that ForgeAgents use to request
image compilation operations.
"""

from .forgeimages_skill import ForgeImagesSkill, ForgeImagesError

__all__ = [
    "ForgeImagesSkill",
    "ForgeImagesError",
]
