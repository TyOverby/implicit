# Problem:

API design for this kind of project is hard.  The programatic API needs to be
nice to use, but it also needs to be powerfull enough to allow for a
graphical tool to be build.


## Scene

Specifically, there needs to be a great Scene api that allows for many
objects to be added, then rendered, and selectively re-rendered.  For
example, changing one object doesn't mean that the entire scene should be
re-rendered.

## Lines (and how lines work)

Implicit surfaces are built off of the idea that everything is a solid.  This
doesn't hold up at the very end of the process though when we want to convert
from an implicit to a cut-out line.

Sometimes this line circles all the way around an object, but other times
it requires a mask.  This mask is different from implicit-based boolean ops
because the mask can only be subtractive (there's no such thing as an
additive mask).  Also, the mask removes parts of the *line*, not parts of
the shape.

Lines can be either a solid outline, or dashed.

Dashed lines have two different modes of interacting with masks.

1. Cut off parts of dashed lines.
2. Don't draw *anything* for a dash that has a part of it masked.

Behavior 1 is closer to what you get for solid lines, but behavior 2 can
be really useful for when you never want a "partially drawn" dash.

# API

Top level elements in the API

```rust
trait Implicit;

enum DrawObject<I: Implicit> {
    Solid(I),
    Line(I),
    DashedLine(I, Vec<f32>, bool)
}

struct Scene {
    new(Vec<DrawObject>) -> Scene;
    render_svg(&self) -> String;
}

enum RenderedType {
    Solid(Vec<f32>),
}
```
