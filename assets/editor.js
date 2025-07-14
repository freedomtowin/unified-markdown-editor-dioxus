


window.getCaretClickPosition = function (element_id = '') {
    let el = document.getElementById(element_id);
    if (el) {{
        let sel = window.getSelection();
        if (sel.rangeCount > 0) {{
            let range = sel.getRangeAt(0);
            let preCaretRange = range.cloneRange();
            preCaretRange.selectNodeContents(el);
            preCaretRange.setEnd(range.endContainer, range.startPosition);
            return range.startOffset;
        }}
    }}
    return 0;
}

window.getElementText = function (element_id = '') {
    // Get the element by ID
    const element = document.getElementById(element_id);

    return element.textContent;

}

window.clearElementText = function (element_id = '', text_content = '') {
    // Get the element by ID
    const element = document.getElementById(element_id);

    element.textContent = text_content;
}

window.clearElementTextWithPosition = function (element_id = '', text_content = '', caretPos) {
    // Get the element by ID
    const element = document.getElementById(element_id);

    element.textContent = text_content;


    // Restore the caret position
    if ('setSelectionRange' in element) {
        if (typeof caretPos === 'number') {
            element.setSelectionRange(caretPos, caretPos);
        } else {
            console.warn(`caretPos should be a number for input elements.`);
        }
    } else if (element.isContentEditable) {
        const selection = window.getSelection();
        const range = document.createRange();

        // Try to get a text node to place the caret
        let node = element.firstChild;
        while (node && node.nodeType !== Node.TEXT_NODE) {
            node = node.firstChild;
        }

        if (!node) {
            // Fallback to the element itself if no text node
            node = element;
            caretPos = Math.min(caretPos, element.textContent.length);
        }

        const safePos = Math.min(caretPos, node.length || 0);
        try {
            range.setStart(node, safePos);
            range.collapse(true);
            selection.removeAllRanges();
            selection.addRange(range);
        } catch (e) {
            console.warn(`Could not set caret position:`, e);
        }
    }


    return '';
}


window.focusElement = function (element_id = '') {
    const el = document.getElementById(element_id);
    if (el) {
      el.focus();
    }
  }

window.focusElementAndSetCaret = function (elementId, caretPos) {
    const el = document.getElementById(elementId);
    if (!el) {
      console.warn(`Element with id '${elementId}' not found.`);
      return;
    }

    el.focus();
  
    // Handle input/textarea
    if ('setSelectionRange' in el) {
      if (typeof caretPos === 'number') {
        el.setSelectionRange(caretPos, caretPos);
      } else {
        console.warn(`caretPos should be a number for input elements.`);
      }
      return;
    }
  
    // Handle contenteditable
    if (el.isContentEditable) {
      const selection = window.getSelection();
      const range = document.createRange();
  
      // Try to get a text node to place the caret
      let node = el.firstChild;
      while (node && node.nodeType !== Node.TEXT_NODE) {
        node = node.firstChild;
      }
  
      if (!node) {
        // Fallback to the element itself if no text node
        node = el;
        caretPos = 0;
      }
  
      const safePos = Math.min(caretPos || 0, node.length || 0);
      try {
        range.setStart(node, safePos);
        range.collapse(true);
        selection.removeAllRanges();
        selection.addRange(range);
      } catch (e) {
        console.warn(`Could not set caret position:`, e);
      }
    }
  };