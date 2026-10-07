# Design

Native GPUI product interface. A designer works at a desktop display with Fusion muscle memory; the user explicitly requests one dark theme, Zed's Gruvbox Dark.

## Palette

Use the roles from Zed's assets/themes/gruvbox/gruvbox.json, Gruvbox Dark:
editor/viewport #282828, panels #3a3735, hover #494340, selection #5b524c,
border #5b534d, text/icon #fbf1c7, muted #c5b597, disabled #998b78,
error #fb4a35, warning #f9bd2f, success #b7bb26, blue #83a598.
Source: https://github.com/zed-industries/zed/blob/main/assets/themes/gruvbox/gruvbox.json

## Structure

A compact top row has file/undo actions and one closable tab per open design, plus a new-design button. Under it, a large workspace dropdown selects Solid, Sketch or Drawing, beside grouped tool ribbons with pinnable commands. A collapsible document tree floats over the upper left of the model viewport. Contextual editors appear only when requested. The viewport's upper right holds an interactive orientation cube and view/grid controls. A persistent construction timeline with a draggable evaluation marker is at the bottom; parameter edit undo and redo remain in memory only. No bottom navigation toolbar.

## Typography and controls

System sans, 14 px interface text, minimal section labels. Icon buttons are 40 px targets with 28 px supplied LibreCAD SVGs, subtle hover backgrounds, selected states, tooltips and keyboard focus. Menus contain tool names and explicit Not implemented status for unfinished tools. Panels have one-pixel separators, no nested cards.

## Assets

Use the provided librecad_svg_icons with their original geometry. Adapt existing color slots at runtime for dark-theme contrast; preserve the source SVGs. Reuse existing artwork as placeholders when no exact feature icon exists; do not author new icons. Its readme states the assets are CC0.
