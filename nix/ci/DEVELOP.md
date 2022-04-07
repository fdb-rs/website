# CI Development Notes

The CI system is designed around NixOS containers and runs using
`podman` on GitHub. It makes a number of assumptions that is
documented here. If you make changes, please update the new
assumptions here.

1. Within the container, we use uid/gid of `1001/121` to run workflow
   steps. This maps to username/groupname `runner/docker`. The uid/gid
   and username/groupname is the same on both the host ubuntu virtual
   machine and the container.

2. `GITHUB_WORKSPACE` is bind-mounted to `/home/runner/website`.
