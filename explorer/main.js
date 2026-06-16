import {EditorState} from "@codemirror/state"
import {EditorView, keymap} from "@codemirror/view"
import {defaultKeymap} from "@codemirror/commands"
import init, { compile_to_js } from "compiler"

await init();

let rhsStartState = EditorState.create({
  doc: "Hello World",
  extensions: [keymap.of(defaultKeymap)]
});

let rhsView = new EditorView({
  state: rhsStartState,
  parent: document.getElementById("rhs-editor")
});

const lhsUpdateListener = EditorView.updateListener.of((update) => {
  if (update.docChanged) {
    const newText = update.state.doc.toString();
    let result = compile_to_js(newText);
    if (result.errors.length == 0) {
        rhsView.dispatch({
            changes: {
                from: 0, 
                to: rhsView.state.doc.length, 
                insert: result.js
            }
        });
    } else {
        console.log(result.errors);
    }
    result.free();
  }
});

let lhsStartState = EditorState.create({
  doc: "Hello World",
  extensions: [keymap.of(defaultKeymap), lhsUpdateListener]
});

let lhsView = new EditorView({
  state: lhsStartState,
  parent: document.getElementById("lhs-editor")
});


