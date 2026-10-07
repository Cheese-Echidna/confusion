# Solid / Create

Every Solid/Create entry opens a working editor. Apply (or Enter) evaluates exact
OCCT geometry before committing the feature; errors leave the current document
unchanged. Features appear in the body browser and construction timeline. Click a
feature to edit its inputs. Undo/redo restores feature intent, and `.con` version 6
stores the inputs and regenerates the geometry on open; older versions still load.

Length fields accept units, arithmetic, and named length parameters. Bare numeric
lengths default to millimetres and angles to degrees. Counts, tooth counts, and
principal-plane selectors are dimensionless parameters. The Parameters panel can
edit all expressions, including these scalar values.

| Tool | Current operation |
| --- | --- |
| Box, cylinder, sphere, torus | Exact new solid at the origin; specify dimensions. |
| Revolve | Closed XY profile around a Y axis at the specified X; up to 360°. Keep the profile on one side of the axis. |
| Sweep | Closed XY profile swept along a straight XYZ vector. |
| Loft | Two different closed XY sketches separated by the specified Z distance; one outer wire per sketch. |
| Rib | Vertical rectangular reinforcement positioned by XYZ, joined to the target. |
| Web | Horizontal rectangular reinforcement positioned by XYZ, joined to the target. |
| Emboss | Closed XY sketch raised from a specified Z height and joined to the target; planar support. |
| Hole | Cylindrical cut starting at XYZ and extending along positive Z. |
| Thread | Modeled rounded helical groove cut around the origin Z axis; set helix radius, pitch, turns, and section radius. No standard thread-size catalog. |
| Coil | Circular section swept along a helix around the origin Z axis. |
| Pipe | Straight hollow cylinder along Z with outer radius, wall thickness, and length. |
| Rectangular pattern | Body copies in an XY grid. Counts include the original. |
| Circular pattern | Body copies rotated about the origin Z axis. Full spans avoid a duplicate at 360°. |
| Pattern on path | Body copies distributed along a straight XYZ displacement. |
| Mirror | Body copy across YZ (0), XZ (1), or XY (2), at a chosen plane offset. |
| Thicken | Closed wall around a solid, from its outward offset minus its original volume. |
| Boundary fill | Common enclosed cell of two overlapping solids; consumes both inputs. |
| Gear | Spur gear with sampled involute teeth at 20° pressure angle, module, tooth count, thickness, and optional bore. |

Profile tools require a closed, conflict-free XY sketch. Select boundary curves
before opening the tool if a sketch contains several regions. The second loft
sketch must contain one unambiguous region. Body tools provide explicit input-body
buttons. Joined/cut/replaced bodies disappear from the list of available inputs;
patterns and mirrors leave their source body intact. A pattern feature groups its
copies as a compound, so subsequent operations requiring a single solid may need
a different input.

The evaluator currently regenerates extrusions first, then Create features in
stored dependency order, then Modify edits. Create features can reference previous
Create bodies or extrusions. Their outputs do not expose semantic extrusion cap
attachments for face-supported sketches. Complex freeform paths, curved embossing,
surface thickening, and general boundary-cell selection remain extensions beyond
the operations described above.
