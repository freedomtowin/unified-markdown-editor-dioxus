# unified-markdown-editor-dioxus

`unified-markdown-editor-dioxus` is a Rust app built with Dioxus to simulate a WYSIWYG unified-rich-text editor for Markdown text.

The way this currently works is that event handlers are logged into a state using a content-editable `div`.

The HTML is rendered by parsing the input text and outputting Dioxus VNodes. The editor is then updated with the HTML on events.

Then JS functions are used to parse the HTML to find where the cursor positions should be. Currently, it only works with Headings and paragraphs.

There are a few text editor functions built-in like Backspace/Delte and a few short cuts, but they don't currently work as intended.

TODO:
1. Fix bold and italic cursor positions
2. Allow for rendering images (and have the url/text be editable on click)
3. Add highlight selection for deletion/paste
4. Fix up/down arrows

Use at your own risk!