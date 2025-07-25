# unified-markdown-editor-dioxus

`unified-markdown-editor-dioxus` is a Rust app built with Dioxus to simulate a WYSIWYG unified-rich-text editor for Markdown.


#### Design Strategy 

Content editable `div` are parsed into a sparse matrix of lines/inline cells, where each cell can be styled.

Updates to the text are done sequentially in an event loop for every keydown event.

#### Input Handlers

  Enter Key (handle_enter_key)

  - Capability: Splits text at cursor position, creates new row
  - Logic: Takes text before cursor, keeps it in current cell; moves text after cursor + remaining columns to new
  row
  - DOM Operations: update_text, create_row, update_text_cursor
  - Edge Cases: Handles splitting at any position within text

  #### Backspace (handle_backspace)

  - Capability: Complex backspace logic with multiple scenarios
  - Scenarios:
    a. pos=0, i>0, j==0: Merge current row with previous row, delete current row
    b. pos=0, i>0, j>0: Move to previous column (currently incomplete)
    c. pos=1, j>0: Merge current cell with previous cell, delete character
    d. pos=0: Regular text sync
    e. Default: Normal character deletion
  - DOM Operations: update_row, delete_row, update_text, update_text_cursor
  - Edge Cases: Row merging, column merging, text concatenation

#### Arrow Keys

  - Left Arrow (handle_left_arrow): Navigate left with cell/row boundaries
  - Right Arrow (handle_right_arrow): Navigate right with cell/row boundaries
  - Up Arrow (handle_up_arrow): Navigate up maintaining column position
  - Down Arrow (handle_down_arrow): Navigate down maintaining column position
  - Edge Cases: Boundary handling, text length preservation, focus management

 #### Character Input (handle_character_input)

  - Capability: Insert characters at cursor position
  - DOM Operations: update_text_cursor
  - Edge Cases: Cursor position validation, text insertion

 #### DOM Update Operations

 #### Text Operations

  - update_text: Update cell content without cursor
  - update_text_cursor: Update cell content with cursor positioning
  - focus_element: Focus element with cursor position

 #### Row Operations

  - create_row: Create new row and renumber subsequent rows
  - delete_row: Delete row and renumber subsequent rows
  - update_row: Replace row content without affecting other rows

 #### Cell Operations 

  - create_cell: Create individual cell with specific index
  - delete_element: Remove specific element from DOM and raw_text

 #### System Operations

  - internal_process: Boolean signal to prevent recursive update_syntax calls

 #### JavaScript Functions

 #### Caret/Focus Management

  - getCaretClickPosition: Get cursor position within element
  - focusElement: Basic element focus
  - focusElementAndSetCaret: Focus with cursor positioning
  - clearElementTextWithPosition: Update text with cursor positioning

 #### DOM Manipulation

  - deleteElement: Remove element from DOM
  - clearElementText: Clear element content
  - getElementText: Get element text content

 #### Row/Cell Management

  - deleteRow: Delete row and renumber subsequent elements
  - createRow: Create new row with proper insertion and renumbering
  - createCell: Create individual cell in specified row
  - updateRow: Replace entire row content without affecting other rows

 #### Edge Cases & Special Handling

 #### Index Management

  - Row/column indexing with textarea-{i}-{j} format
  - Automatic renumbering after row operations
  - Bounds checking for array access

 #### State Synchronization

  - editor.raw_text vs visual DOM consistency
  - internal_process signal prevents recursive updates
  - Queue-based DOM updates with async processing

 #### Text Processing

  - Empty element removal from multi-element rows
  - Character insertion/deletion at specific positions
  - Text splitting and merging across cells/rows

 #### Focus & Cursor

  - Cross-browser cursor positioning
  - ContentEditable vs input element handling
  - Focus management during DOM updates

 #### Concurrency

  - Spawn-based async operations
  - DOM update queue processing
  - Wait mechanisms for pending updates

 #### Error Handling

  - Element existence validation
  - Bounds checking for text operations
  - Fallback behaviors for cursor positioning

  #### System Architecture

  - Signal-based state management with use_signal
  - Coroutine-based async processing for DOM updates
  - Event-driven keyboard handling with preventDefault
  - Queue-based DOM operation batching
  - JavaScript interop via document::eval

  The system handles complex text editing scenarios with proper state synchronization between Rust backend and JavaScript DOM manipulation.

 #### TODO:
1. Add additional keydown events like TAB, DELETE, etc
2. Fix Copy/Paste Operations
3. Many, many more TODOs

Use at your own risk!