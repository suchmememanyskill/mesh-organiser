window.addEventListener("click", (event) => {
    if (event.ctrlKey || event.altKey || event.shiftKey) {
        event.preventDefault();
    }
});

window.addEventListener("auxclick", (event) => {
    if (event.button === 1) {
        event.preventDefault();
    }
});

function runInEveryRoot(node = document) {
    node.addEventListener("click", (event) => {
        [event.target, event.target.parentElement].forEach((el) => {
            if (el.nodeName === "A" && el.getAttribute("target") === "_blank") {
                el.removeAttribute("target");
            }
        });
    });

    node.open = null;

    const treeWalker = document.createTreeWalker(node, NodeFilter.SHOW_ELEMENT, null, false);
    while (treeWalker.nextNode()) {
        const el = treeWalker.currentNode;
        if (el.shadowRoot) {
            runInEveryRoot(el.shadowRoot);
        }
    }
}

setTimeout(() => {
    runInEveryRoot();
}, 1000);

document.body.setAttribute('tauri-inject', 'true');