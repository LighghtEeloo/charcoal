import os
from pathlib import Path

repo_proj = "charcoal"
repo_aur = "charcoal-aur"

root_parent = Path(os.getcwd()).parent
# .
# |- <repo_proj> [here you are]
# |- <repo_aur>

pkgbuild = "PKGBUILD"
pkgbuild_proj = root_parent.joinpath(repo_proj, pkgbuild)
pkgbuild_aur = root_parent.joinpath(repo_aur, pkgbuild)

srcinfo = ".SRCINFO"
srcinfo_aur = root_parent.joinpath(repo_aur, srcinfo)

os.system(f"cp -f {pkgbuild_proj} {pkgbuild_aur}")
os.system(f"makepkg -g >> {pkgbuild_aur}")
os.system(f"makepkg --printsrcinfo > {srcinfo_aur}")
os.system(f"rm charcoal-*.tar.gz")
