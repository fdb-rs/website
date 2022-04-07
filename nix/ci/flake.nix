{
  description = "fdb-rs website flake";

  inputs.nixpkgs.url = "nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: {
    website =
      let
        pkgs = import nixpkgs {
          system = "x86_64-linux";
        };

        nix-conf = pkgs.writeTextDir "etc/nix/nix.conf" ''
          sandbox = false
          max-jobs = auto
          cores = 0
          trusted-users = root runner
          experimental-features = nix-command flakes
        '';

        systemd-units = builtins.attrValues (import ./systemd { inherit pkgs; });

        nss-files = import ./nss { inherit pkgs; };
      in
      with pkgs;
      dockerTools.buildImageWithNixDb {
        name = "website";
        tag = "latest";

        contents = [
          (symlinkJoin {
            name = "container-symlinks";
            paths = [
              bashInteractive
              cacert
              coreutils
              curl
              findutils
              git
              glibc.bin
              gnugrep
              gnutar
              gzip
              iproute2
              iputils
              nix-conf
              nixUnstable
              shadow
              systemd
              utillinux
              vim
              which
            ]
            ++ [
              zola
            ]
            ++ systemd-units
            ++ nss-files;
          })
        ];

        runAsRoot = ''
          mkdir -p -m 1777 /tmp

          mkdir -p /usr/bin
          ln -s ${coreutils}/bin/env /usr/bin/env

          touch /etc/machine-id
          mkdir -p /var
          ln -s /run /var/run

          mkdir -p /home/runner/website
          chown -R runner:docker /home/runner

          systemctl enable nix-daemon.socket
        '';

        config = {
          Cmd = [ "/lib/systemd/systemd" ];

          Env = [
            "NIX_SSL_CERT_FILE=${cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
        };
      };
  };
}
