# Adaptives
Adaptive shapes are created with the MVEngine Shape Format. To see the language explanation, go to the Shapes section.
These nicely-scaling shapes have the property of having separate shapes for center, borders and corners. The following diagram illustrates the sub-shapes:

| **tl** | **t** | **tr** |
|--------|-------|--------|
| **l**  | **c** | **r**  |
| **bl** | **b** | **br** |

I cannot be asked to make an actual image, so the table it is.
Anyways, because we have nine separate shapes, all of them have to be exported using their target like in the table.

Remember that each sub-shape is technically its own thing, so for example, don't put the `tr` corner further away from the origin. Think of them as they were in separate files.

The following MSF code creates a hollow rectangle, which will scale nicely.
```
corner = rect[x0y0w2h2];
>corner;
export bl;
export br;
export tr;
export tl;

bar = rect[x0y0w2h2];
>bar;
export t;
export l;
export b;
export r;

export;
```


The following code creates a rounded rectangle:
```
bl = arc[c[x5y5]a90tc3r5t[r180o[x5y5]]];
tl = arc[c[x5y0]a90tc3r5t[r90o[x5y0]]];
tr = arc[c[x0y0]a90tc3r5t[r0o[x0y0]]];
br = arc[c[x0y5]a90tc3r5t[r270o[x0y5]]];

>bl;
apply;
>tl;
apply;
>tr;
apply;
>br;
apply;

r = rect[x0y0w5h5];
>r;
export t;
export l;
export b;
export r;
export c;

>bl;
export bl;

>tl;
export tl;

>tr;
export tr;

>br;
export br;

export;
```