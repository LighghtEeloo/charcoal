import os, sys
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


if __name__ == "__main__":

    def bump_ver(ver):
        os.system(f"cargo set-version {ver}")
        with open(pkgbuild_proj, "r") as f:
            pkgcontent = f.readlines()
        assert pkgcontent[3].startswith("pkgver")
        pkgcontent[3] = f"pkgver={ver}\n"
        with open(pkgbuild_proj, "w") as f:
            f.writelines(pkgcontent)

    def upload_pkg():
        os.system(f"cp -f {pkgbuild_proj} {pkgbuild_aur}")
        os.system(f"makepkg -g >> {pkgbuild_aur}")
        os.system(f"makepkg --printsrcinfo > {srcinfo_aur}")
        os.system(f"rm charcoal-*.tar.gz")

    {
        "bump": lambda: bump_ver(sys.argv[2]),
        "upload": lambda: upload_pkg(),
    }[sys.argv[1]]
