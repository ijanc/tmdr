# tmdr example

This file exercises every element that **tmdr** supports.

## Paragraphs

A short paragraph.

A longer paragraph that contains enough words to verify that line wrapping
works correctly when the text exceeds the configured column width. It should
break at word boundaries and never split a word in half.

## Inline formatting

This is **bold**, this is *italic*, and this is `inline code`.
Mixing **bold with *italic* inside** also works.

## Lists

Unordered:

- First item
- Second item with **bold** and *italic*
- Third item that is long enough to test wrapping inside a list item with
  a hanging indent

Ordered:

1. One
2. Two
3. Three with `inline code` in it

## Code

```
#include <stdio.h>

int
main(void)
{
	printf("hello, world\n");
	return 0;
}
```

## Links

Visit [OpenBSD](https://www.openbsd.org) for more info.
A [link with **bold**](https://example.com) inside a paragraph.

## Headers

### Level 3

#### Level 4

##### Level 5

###### Level 6

---

End of example.
