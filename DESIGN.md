# Design

Native GPUI product interface. A designer works at a desktop display with Fusion muscle memory; the user explicitly requests one dark theme, Zed's Gruvbox Dark.

## Palette

Use the roles from Zed's assets/themes/gruvbox/gruvbox.json, Gruvbox Dark:
editor/viewport #282828, panels #3a3735, hover #494340, selection #5b524c,
border #5b534d, text/icon #fbf1c7, muted #c5b597, disabled #998b78,
error #fb4a35, warning #f9bd2f, success #b7bb26, blue #83a598.
Source: https://github.com/zed-industries/zed/blob/main/assets/themes/gruvbox/gruvbox.json

## Structure

A compact top row has file/undo actions and workspace tabs. Under it, grouped mode-specific tool ribbons expose feature menus. A collapsible document tree sits to the left of the model viewport. Contextual editors appear only when requested. The viewport's upper right holds an interactive orientation cube and view/grid controls. A session timeline is at the bottom. No bottom navigation toolbar.

## Typography and controls

System sans, 12–13 px interface text, minimal section labels. Icon buttons are 30–36 px targets with 18–22 px supplied LibreCAD SVGs, subtle hover backgrounds, selected states, tooltips and keyboard focus. Menus contain tool names and explicit Not implemented status for unfinished tools. Panels have one-pixel separators, no nested cards.

## Assets

Use the provided librecad_svg_icons unchanged. Reuse existing artwork as placeholders when no exact feature icon exists; do not author new icons. Its readme states the assets are CC0.
