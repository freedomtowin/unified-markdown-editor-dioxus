function copyPreviewToEditor() {
    const preview = document.getElementById("preview");
    const editor = document.getElementById("editor");
    if (!preview || !editor) return "";

    // 1) Save the current cursor position in the "raw text" sense,
    //    using your existing getCaretClickPosition() function.
    let offset = 0;
    const selection = window.getSelection();
    if (selection.rangeCount > 0 && editor.contains(selection.focusNode)) {
        offset = getCaretClickPosition();
    }

    // 2) Overwrite the editor’s content with preview’s HTML
    editor.innerHTML = preview.innerHTML;

    // 3) Restore the caret in the editor at the same raw offset
    if (offset > 0) {
        insertCursorAtPosition(offset);
    }

    // 4) Return editor's text (or any string representation you want)
    return editor.innerText;
}


function getCaretClickPosition() {
    const editor = document.getElementById("editor");
    const selection = window.getSelection();
    if (!selection.rangeCount) {
        console.log('No selection range found');
        return 0;
    }

    const range = selection.getRangeAt(0);
    let offset = 0;
    const treeWalker = document.createTreeWalker(
        editor,
        NodeFilter.SHOW_ALL,
        null
    );

    let previousNode = null;
    console.log('Starting TreeWalker for click position');

    while (treeWalker.nextNode()) {
        const node = treeWalker.currentNode;

        if (node === range.endContainer) {
            if (node.nodeType === Node.TEXT_NODE) {
                offset += range.endOffset;
                console.log('End reached at text node:', { text: node.textContent, endOffset: range.endOffset, totalOffset: offset });
            } else {
                console.log('End reached at non-text node:', { nodeName: node.nodeName, totalOffset: offset });
            }
            break;
        }

        if (node.nodeType === Node.TEXT_NODE) {
            offset += node.textContent.length;
            console.log('Text node processed:', { text: node.textContent, length: node.textContent.length, offset });
        } else if (node.nodeName === 'BR' || (isBlockElement(node) && node.childNodes.length === 1 && node.firstChild.nodeName === 'BR')) {
            offset += 2; // Treat <p><br>\n</p> as \n\n
            console.log('Empty line or BR detected:', { tag: node.nodeName, offset });
        } else if (previousNode && previousNode.parentElement !== node.parentElement && 
                  (isBlockElement(previousNode.parentElement) || isBlockElement(node.parentElement))) {
            offset += 2; // Block separation as \n\n
            console.log('Block separation:', { from: previousNode.parentElement.tagName, to: node.nodeName || node.parentElement.tagName, offset });
        }

        previousNode = node;
    }

    console.log('Final click position:', offset);
    return offset;
}

function insertCursorAtPosition(pos, wasEnterPressed = false) {
    const editor = document.getElementById("editor");
    if (!editor) return;

    // If user just pressed Enter, shift the position by +1
    // (or +2 if your offsets treat blocks or line breaks as +2).
    if (wasEnterPressed) {
        pos -= 0; 
    }

    editor.focus();
    const sel = window.getSelection();
    sel.removeAllRanges();

    const range = document.createRange();
    const result = findTextNodeAndOffset(editor, pos);
    if (result.node) {
        if (result.node.nodeType === Node.TEXT_NODE) {
            range.setStart(result.node, result.offset);
        } else {
            range.setStartBefore(result.node);
        }
        range.collapse(true);
        sel.addRange(range);
    }
}

// Normalize the raw text by replacing Windows-style newlines (\r\n) with Unix-style newlines (\n)
// and replacing &nbsp; (\u00a0) with a single space.
function normalizeText(text) {
    return text.replace(/\r\n/g, '\n').replace(/\u00a0/g, ' ');
}


function isBlockElement(element) {
    const blockElements = ['P', 'DIV', 'BR', 'PRE', 'BLOCKQUOTE', 'H1', 'H2', 'H3', 'H4', 'H5', 'H6'];
    return blockElements.includes(element.tagName);
} 

// Calculate the total length of all text nodes in the editor, including normalized text
function getTotalTextLength(node) {
    let length = 0;
    if (node.nodeType === Node.TEXT_NODE) {
        length += normalizeText(node.textContent).length;
    } else {
        for (let i = 0; i < node.childNodes.length; i++) {
            length += getTotalTextLength(node.childNodes[i]);
        }
    }
    return length;
}



// Find the text node and offset corresponding to the given position
function findTextNodeAndOffset(node, pos) {
    let stack = [node];
    let offset = 0;
    let previousNode = null;

    while (stack.length > 0) {
        let current = stack.shift();

        if (current.nodeType === Node.TEXT_NODE) {
            const normalizedText = normalizeText(current.textContent);
            const textLength = normalizedText.length;

            // Adjust offset based on previous node
            if (previousNode) {
                let parent1 = previousNode.parentElement;
                let parent2 = current.parentElement;

                // Handle inline elements properly
                const isInlineParent1 = parent1 && !isBlockElement(parent1);
                const isInlineParent2 = parent2 && !isBlockElement(parent2);

                if (parent1 !== parent2) {
                    if (!isInlineParent1 || !isInlineParent2) {
                        offset += 2; // Two newlines for block separation
                    } else {
                        offset += 1; // One newline for inline separation
                    }
                } else if (previousNode.nextSibling && previousNode.nextSibling.nodeName === 'BR') {
                    offset += 1; // Single newline for explicit breaks
                }
            }

            if (offset + textLength >= pos) {
                return { node: current, offset: pos - offset };
            }
            offset += textLength;
            previousNode = current;
        } else {
            // Add child nodes in reverse order to maintain correct traversal
            for (let i = current.childNodes.length - 1; i >= 0; i--) {
                stack.unshift(current.childNodes[i]);
            }
        }
    }
    // If no text node is found, return the element itself with offset 0
    return { node: el, offset: 0 };
}
