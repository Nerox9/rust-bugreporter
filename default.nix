{ lib
, stdenv
, nixpkgs ? import <nixpkgs> {}
, pkg-config
, rustPlatform
, gtk4
, libadwaita
, pango
, cairo
, glib
, wrapGAppsHook4
, githubToken ? ""
, githubOwner ? ""
, githubRepo ? ""
}:

rustPlatform.buildRustPackage {
  pname = "bug-reporter";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
    };
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = [
    gtk4
    libadwaita
    pango
    cairo
    glib
  ];

  postInstall = ''
    mkdir -p $out/share/bug-reporter
    cat > $out/share/bug-reporter/config.toml << EOF
[window]
default_width = 400
default_height = 200
application_id = "com.example.bug-reporter"

[github]
token = "${githubToken}"
owner = "${githubOwner}"
repo = "${githubRepo}"
EOF

    # Create symlink to config file in bin directory
    ln -s $out/share/bug-reporter/config.toml $out/bin/config.toml
  '';

  meta = with lib; {
    description = "A GTK4 bug reporting application";
    homepage = "https://github.com/yourusername/bug-reporter";
    license = licenses.mit;
    maintainers = [ ];
    platforms = platforms.unix;
  };
}
