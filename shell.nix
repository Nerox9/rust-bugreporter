let
  nixpkgs = import <nixpkgs> {};
in
nixpkgs.callPackage ./default.nix {
  githubToken = "your-token";
  githubOwner = "your-username";
  githubRepo = "your-repo";
}
