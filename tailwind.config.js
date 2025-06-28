module.exports = {
  content: [
    "./index.html",
    "./src/**/*.rs"
  ],
  theme: {
    extend: {},
  },
  plugins: [
    require("daisyui")
  ],
  // If using Nix-built daisyui, ensure NODE_PATH is set in your build (flake.nix)
}
