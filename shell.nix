with import <nixpkgs> {};
mkShell {
	buildInputs = [
		cargo
		cargo-make
		cargo-edit
		wasm-pack
		# my-rust
		# nodePackages.tailwindcss
		rust-analyzer # IDE
		libiconv curl # native libs
	];
}
