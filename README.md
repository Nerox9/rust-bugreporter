# Bug Reporter

A GTK4-based bug reporting application that allows users to submit bug reports with screenshots directly to GitHub issues.

## Prerequisites

### For conventional building:
- Rust and Cargo (latest stable version)
- GTK4 development libraries
- libadwaita development libraries
- pkg-config
- Other system dependencies:
  - pango
  - cairo
  - glib

#### Installing dependencies on different systems:

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev pkg-config libpango1.0-dev libcairo2-dev libglib2.0-dev
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel pkg-config pango-devel cairo-devel glib2-devel
```

**macOS:**
```bash
brew install gtk4 libadwaita pkg-config pango cairo glib
```

### For Nix building:
- Nix package manager

## Building

### Conventional Build

1. Clone the repository:
```bash
git clone <repository-url>
cd bug-reporter
```

2. Build the project:
```bash
cargo build --release
```

3. Run the application:
```bash
cargo run --release
```

### Nix Build

1. Build with default configuration:
```bash
nix-build
```

2. Build with custom GitHub configuration:
```bash
nix-build --arg githubToken '"your-token"' \
          --arg githubOwner '"your-username"' \
          --arg githubRepo '"your-repo"'
```

## Configuration

The application requires a `config.toml` file with the following structure:

```toml
[window]
default_width = 400
default_height = 200
application_id = "com.example.bug-reporter"

[github]
token = "your-github-token"
owner = "github-username"
repo = "repository-name"
```

### Configuration Methods

1. **Direct file:** Place `config.toml` in the same directory as the executable.

2. **Nix build:** Pass GitHub configuration parameters during build:
   ```bash
   nix-build --arg githubToken '"your-token"' \
             --arg githubOwner '"your-username"' \
             --arg githubRepo '"your-repo"'
   ```
   This will generate the config file automatically in the package's output.

### GitHub Token

To create a GitHub personal access token:
1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Generate a new token with `repo` scope
3. Copy the token and add it to your configuration

## Features

- Multi-step bug report wizard
- System information collection (MAC address, hostname)
- Screenshot attachment support
- Direct GitHub issue creation
- GTK4 modern user interface

## Development

To contribute to the project:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## NixOS Integration

To use this package in your NixOS configuration, you can add it to your system packages. Here's how to do it:

1. Add the package to your NixOS configuration (`configuration.nix`):

```nix
{ config, pkgs, ... }:

let
  bug-reporter = pkgs.callPackage /path/to/bug-reporter/default.nix {
    githubToken = "your-github-token";
    githubOwner = "your-github-username";
    githubRepo = "your-repository";
  };
in
{
  environment.systemPackages = with pkgs; [
    bug-reporter
  ];
}
```

2. Or, create a module for more flexibility:

```nix
# bug-reporter.nix
{ config, lib, pkgs, ... }:

with lib;

let
  cfg = config.services.bug-reporter;
in {
  options.services.bug-reporter = {
    enable = mkEnableOption "bug reporter application";
    
    githubToken = mkOption {
      type = types.str;
      description = "GitHub personal access token";
    };
    
    githubOwner = mkOption {
      type = types.str;
      description = "GitHub repository owner";
    };
    
    githubRepo = mkOption {
      type = types.str;
      description = "GitHub repository name";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [
      (pkgs.callPackage /path/to/bug-reporter/default.nix {
        inherit (cfg) githubToken githubOwner githubRepo;
      })
    ];
  };
}
```

Then in your `configuration.nix`:

```nix
{ config, ... }:

{
  imports = [ ./bug-reporter.nix ];
  
  services.bug-reporter = {
    enable = true;
    githubToken = "your-github-token";
    githubOwner = "your-github-username";
    githubRepo = "your-repository";
  };
}
```

3. After adding the configuration, rebuild your NixOS system:

```bash
sudo nixos-rebuild switch
```

The bug reporter application will be available in your system PATH after rebuilding.

## License

MIT License - See LICENSE file for details
