#!/usr/bin/env bash
# Wrapper to run tailwindcss in the current shell (Nix devShell aware)
exec tailwindcss -i tailwind.input.css -o tailwind.output.css
