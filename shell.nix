{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    clippy
    pkg-config
    openssl
  ];

  shellHook = ''
    echo ""
    echo "Drift Development Environment"
    echo "=============================="
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
    echo "Commands:"
    echo "  cargo build    - Build the project"
    echo "  cargo run      - Run drift"
    echo "  cargo test     - Run tests"
    echo "  cargo clippy   - Lint"
    echo ""
  '';
}
