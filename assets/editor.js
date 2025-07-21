


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

window.deleteElement = function (element_id = '') {
    const container = document.getElementById('container');
    container.blur();
    const element = document.getElementById(element_id);
    if (element) {
        element.remove();
        console.log(`Deleted element: ${element_id}`);
    } else {
        console.warn(`Element with id '${element_id}' not found`);
    }
};


window.clearElementText = function (element_id = '', text_content = '') {
    // Get the element by ID
    const element = document.getElementById(element_id);
    element.blur()
    element.textContent = text_content;
    element.blur()

}

window.clearElementTextWithPosition = function (element_id = '', text_content = '', caretPos) {
    const element = document.getElementById(element_id);
    if (!element) return '';

    // Blur all focusable elements in the container (if specified)
    
    const container = document.getElementById('container');
    if (container) {
        const focusables = container.querySelectorAll('div.base-paragraph');
        focusables.forEach(el => {
            if (el !== element) el.blur();
        });
    }


    // Set the text content
    element.textContent = text_content;

    // Focus the element
    element.focus();

    // Restore the caret position
    if ('setSelectionRange' in element && typeof caretPos === 'number') {
        element.setSelectionRange(caretPos, caretPos);
    } else if (element.isContentEditable) {
        const selection = window.getSelection();
        const range = document.createRange();

        // Get the first text node inside the element
        let node = element.firstChild;
        while (node && node.nodeType !== Node.TEXT_NODE) {
            node = node.firstChild;
        }

        if (!node) {
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
            console.warn('Could not set caret position:', e);
        }
    }

    return '';
};


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

window.deleteRow = function (row_id = '', newRowsData = null) {
    const row = document.getElementById(row_id);
    if (row) {
        // Extract row index from row_id (format: textrow-N)
        const deletedRowIndex = parseInt(row_id.split('-')[1]);
        const container = document.getElementById('container');
        
        // Remove the row
        row.remove();
        
        if (container) {
            // Renumber all subsequent rows and their cells
            const allRows = container.querySelectorAll('[id^="textrow-"]');
            for (let remainingRow of allRows) {
                const currentIndex = parseInt(remainingRow.id.split('-')[1]);
                if (currentIndex > deletedRowIndex) {
                    const newRowId = `textrow-${currentIndex - 1}`;
                    remainingRow.id = newRowId;
                    
                    // Update all cells in this row
                    const cells = remainingRow.querySelectorAll('[id^="textarea-"]');
                    for (let cell of cells) {
                        const parts = cell.id.split('-');
                        if (parts.length === 3) {
                            const oldCellIndex = parseInt(parts[1]);
                            if (oldCellIndex === currentIndex) {
                                cell.id = `textarea-${currentIndex - 1}-${parts[2]}`;
                                
                                // Update cell content if new data is provided
                                if (newRowsData && newRowsData[currentIndex - 1 - deletedRowIndex]) {
                                    const rowData = newRowsData[currentIndex - 1 - deletedRowIndex];
                                    const colIndex = parseInt(parts[2]);
                                    if (rowData[colIndex] !== undefined) {
                                        cell.textContent = rowData[colIndex];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        console.log(`Deleted row: ${row_id}`);
    } else {
        console.warn(`Row with id '${row_id}' not found`);
    }
};

window.createRow = function (row_id = '', insertAfter = null) {
  const container = document.getElementById('container');
  if (!container) {
      console.warn('Container not found');
      return null;
  }

  // Extract row index from row_id (format: textrow-N)
  const newRowIndex = row_id ? parseInt(row_id.split('-')[1]) : 0;

  // Check for existing row with the same ID and remove it
  const existingRow = document.getElementById(row_id);
  if (existingRow) {
      existingRow.remove();
      console.log(`Removed existing row: ${row_id}`);
  }

  // Create new row
  const newRow = document.createElement('div');
  newRow.id = row_id;
  newRow.style.cssText = 'display: flex; flex-direction: row; gap: 0; flex-wrap: wrap; font-size: 0;';

  // Insert the new row
  if (insertAfter) {
      const afterElement = document.getElementById(insertAfter);
      if (afterElement && afterElement.nextSibling) {
          container.insertBefore(newRow, afterElement.nextSibling);
      } else {
          container.appendChild(newRow);
      }
  } else {
      container.appendChild(newRow);
  }

  // Renumber all rows to ensure consecutive IDs
  const allRows = Array.from(container.querySelectorAll('[id^="textrow-"]')).sort((a, b) => {
      const aIndex = parseInt(a.id.split('-')[1]) || 0;
      const bIndex = parseInt(b.id.split('-')[1]) || 0;
      return aIndex - bIndex;
  });

  // Update row IDs and their cells
  for (let i = 0; i < allRows.length; i++) {
      const row = allRows[i];
      const currentIndex = parseInt(row.id.split('-')[1]) || 0;
      const newRowId = `textrow-${i}`;
      
      if (row.id !== newRowId) {
          row.id = newRowId;
          // Update all cells in this row
          const cells = row.querySelectorAll('[id^="textarea-"]');
          for (let cell of cells) {
              const parts = cell.id.split('-');
              if (parts.length === 3) {
                  cell.id = `textarea-${i}-${parts[2]}`;
              }
          }
      }
  }

  // Remove empty rows (except the new row if it's empty)
  for (let row of allRows) {
      if (row.id !== row_id && row.children.length === 0) {
          row.remove();
          console.log(`Removed empty row: ${row.id}`);
      }
  }

  console.log(`Created row: ${row_id}`);
  return newRow;
};

window.createRow = function (row_id = '', insertAfter = null) {
  const newRow = document.createElement('div');
  newRow.id = row_id;
  newRow.style.cssText = 'display: flex; flex-direction: row; gap: 0; flex-wrap: wrap; font-size: 0;';
  
  const container = document.getElementById('container');
  if (container) {
      // Extract row index from row_id (format: textrow-N)
      const newRowIndex = parseInt(row_id.split('-')[1]);
      
      if (insertAfter) {
          const afterElement = document.getElementById(insertAfter);
          if (afterElement && afterElement.nextSibling) {
              container.insertBefore(newRow, afterElement.nextSibling);
          } else {
              container.appendChild(newRow);
          }
      } else {
          container.appendChild(newRow);
      }
      
      // Renumber all subsequent rows and their cells
      const allRows = container.querySelectorAll('[id^="textrow-"]');
      for (let row of allRows) {
          const currentIndex = parseInt(row.id.split('-')[1]);
          if (currentIndex > newRowIndex) {
              const oldRowId = row.id;
              const newRowId = `textrow-${currentIndex + 1}`;
              row.id = newRowId;
              
              // Update all cells in this row
              const cells = row.querySelectorAll('[id^="textarea-"]');
              for (let cell of cells) {
                  const parts = cell.id.split('-');
                  if (parts.length === 3) {
                      const oldCellIndex = parseInt(parts[1]);
                      if (oldCellIndex === currentIndex) {
                          cell.id = `textarea-${currentIndex + 1}-${parts[2]}`;
                      }
                  }
              }
          }
      }
      
      console.log(`Created row: ${row_id}`);
  } else {
      console.warn('Container not found');
  }
  return newRow;
};

window.createCell = function (element_id = '', row_id = '', text_content = '', cellStyle = '') {
    const cell = document.createElement('div');
    cell.id = element_id;
    cell.contentEditable = true;
    cell.className = 'base-paragraph';
    cell.style.cssText = cellStyle;
    cell.textContent = text_content;
    
    const row = document.getElementById(row_id);
    if (row) {
        row.appendChild(cell);
        console.log(`Created cell: ${element_id} in row: ${row_id}`);
    } else {
        console.warn(`Row with id '${row_id}' not found`);
    }
    return cell;
};

window.updateRow = function (row_id = '', rowData = []) {
    const row = document.getElementById(row_id);
    if (!row) {
        console.warn(`Row with id '${row_id}' not found`);
        return;
    }
    
    // Clear existing cells in the row
    row.innerHTML = '';
    
    // Create new cells with the provided data
    for (let col_idx = 0; col_idx < rowData.length; col_idx++) {
        const cell = document.createElement('div');
        cell.id = `textarea-${row_id.split('-')[1]}-${col_idx}`;
        cell.contentEditable = true;
        cell.className = 'base-paragraph';
        // cell.style.cssText = 'display: flex; flex-direction: row; gap: 0; flex-wrap: wrap; font-size: 0;';
        cell.textContent = rowData[col_idx];
        row.appendChild(cell);
    }
    
    console.log(`Updated row: ${row_id} with ${rowData.length} columns`);
};