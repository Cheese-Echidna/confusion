# Design

Native GPUI product interface. A designer works at a desktop display with Fusion muscle memory; the user explicitly requests one dark theme, Zed's Gruvbox Dark.

## Palette

Use the roles from Zed's assets/themes/gruvbox/gruvbox.json, Gruvbox Dark:
editor/viewport #282828, panels #3a3735, hover #494340, selection #5b524c,
border #5b534d, text/icon #fbf1c7, muted #c5b597, disabled #998b78,
error #fb4a35, warning #f9bd2f, success #b7bb26, blue #83a598.
Source: https://github.com/zed-industries/zed/blob/main/assets/themes/gruvbox/gruvbox.json

## Structure

A compact top row has file/undo actions and one closable tab per open design, plus a new-design button. Under it, a large workspace dropdown selects Solid, Sketch or Drawing, beside grouped tool ribbons with pinnable commands. A transparent document tree floats over the upper left of the model viewport, with compact disclosure rows and indented children. Contextual editors are compact floating boxes inside the viewport, with Tab/Shift-Tab field navigation, Enter confirmation and Escape cancellation. Dimension values open beside their placed annotation. The viewport's upper right holds an interactive orientation cube and view/grid controls. A persistent construction timeline with a draggable evaluation marker is at the bottom; parameter edit undo and redo remain in memory only. No bottom navigation toolbar.

## Typography and controls

System sans, 14 px interface text, minimal section labels. Icon buttons are 40 px targets with 28 px supplied FreeCAD flat SVGs, subtle hover backgrounds, selected states, tooltips and keyboard focus. Menus contain tool names and explicit Not implemented status for unfinished tools. Panels have one-pixel separators, no nested cards.

## Assets

Use the provided `assets/icons/freecad flat` SVGs with their original geometry and colors. Tool-specific FreeCAD artwork is mapped directly for sketch and solid commands. Legacy semantic aliases in shell controls resolve to FreeCAD artwork; no LibreCAD SVGs are loaded. Reuse supplied artwork where an exact feature icon is unavailable.
