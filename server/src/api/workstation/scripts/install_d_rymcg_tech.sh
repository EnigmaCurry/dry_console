# # Install d.rymcg.tech
#
# d.rymcg.tech is a configuration and deployment environment for Docker (docker-compose).
#
# This script will clone its [git
# repository](https://github.com/EnigmaCurry/d.rymcg.tech) to your
# workstation.

# var: ROOT_DIR="~/git/vendor/enigmacurry/d.rymcg.tech" the directory path where to clone d.rymcg.tech
# help: ROOT_DIR Example: ~/git/vendor/enigmacurry/d.rymcg.tech
# help: ROOT_DIR The chosen directory (and its parents) will be created automatically.

check_var ROOT_DIR
ROOT_DIR=$(expand_home_dir "${ROOT_DIR}")
debug_var ROOT_DIR
D_RYMCG_TECH_REPO=https://github.com/EnigmaCurry/d.rymcg.tech.git

seq 100

test -e ${ROOT_DIR} && \
    stderr "Error: Directory already exists: ${ROOT_DIR}" && \
    fault  "You must specify a non-existant path for ROOT_DIR."

git clone "${D_RYMCG_TECH_REPO}" "${ROOT_DIR}"
