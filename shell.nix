let
	sources = import ./nix/sources.nix {};
	pkgs = import <nixpkgs> {
		overlays = [
			# (import sources.nixpkgs-mozilla)
			# (builtins.trace "FENIX ${sources.fenix}" 
			(import "${sources.fenix}/overlay.nix")
			# )
		];
	};
	my-rust = pkgs.fenix.combine [
		(pkgs.fenix.stable.withComponents ["cargo" "rustc" "rust-src"])
		(pkgs.fenix.targets.wasm32-unknown-unknown.stable.rust-std)
	];
in
with pkgs;
mkShell {
	# RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
	buildInputs = [
		cargo cargo-make
		wasm-pack
		my-rust
		nodePackages.tailwindcss
		rust-analyzer # IDE
		libiconv curl # native libs
		] ++ (
		lib.optionals stdenv.isDarwin (with darwin.apple_sdk; [
			frameworks.Security
			frameworks.CoreServices
			frameworks.CoreFoundation
			frameworks.Foundation
			frameworks.AppKit
		])
	);
}
