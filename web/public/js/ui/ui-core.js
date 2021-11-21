function test() {
    return "Hello World!"
}


class UiCore {
    static customTags = [];

    /**
     * Register a custom tag
     * @type {(tagName: string, tagManager: function) => void}
     * @param tagName Name of the tag
     * @param tagManager Function which will apply code to custom tags
     */
    static registerTag(tagName, tagManager) {
        const tags = document.getElementsByTagName(tagName);
        for (let tag of tags) {
            tagManager(tag);
        }
        this.customTags.push({tagName: tagName, tagManager: tagManager});
    }

    /**
     * Add an element in another element
     * @type {(parent: HTMLElement, element: HTMLElement) => void}
     * @param parent The parent element to append child
     * @param element The child to add
     */
    static appendChild(parent, element) {
        const tagName = element.tagName.toLowerCase();
        this.customTags.forEach(customTag => {
            if (customTag.tagName.toLowerCase() === tagName) customTag.tagManager(element);
        });
        parent.appendChild(element);
    }

    static get mobile() {
        return /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini|SamsungBrowser/i.test(navigator.userAgent);
    }

    static get dark() {
        if (!localStorage.getItem("dark")) localStorage.setItem("dark", "0");
        else return localStorage.getItem("dark") === "1";
    }

    static setDarkAutoDestruct() {
        MutationObserver = window.MutationObserver || window.WebKitMutationObserver;

        const observer = new MutationObserver(function(mutations, observer) {
            const recursive = (node) => {
                if (node.classList.contains("ui") && !node.classList.contains("inverted")) node.classList.add("inverted");
                for (let i = 0; i < node.children.length; i++) {
                    recursive(node.children[i]);
                }
            }
            mutations.forEach(mutation => {
                try {
                    mutation.addedNodes.forEach(node => {
                        try {
                            if (node.classList.contains("ui") && !node.classList.contains("inverted")) {
                                node.classList.add("inverted");
                            } else if (node.classList.contains("pusher"))
                                node.style.background = "#080808";
                            recursive(node);
                        } catch (e) {}
                    });
                } catch (e) {}
            });
        });

        observer.observe(document, {
            subtree: true,
            childList: true
        });

        window.addEventListener("load", () => {
            document.body.style.background = "#080808";
            document.children[0].style.background = "#080808";
            const recursive = (node) => {
                if (node.classList.contains("ui") && !node.classList.contains("inverted")) node.classList.add("inverted");
                for (let i = 0; i < node.children.length; i++) {
                    recursive(node.children[i]);
                }
            }
            recursive(document.body);
        });
        UiCore.setDarkAutoDestruct = null;
    }

    static switchTheme() {
        UiCore.dark? localStorage.setItem("dark", "0"): localStorage.setItem("dark", "1");
        document.location.reload();
    }
}

if (UiCore.dark) UiCore.setDarkAutoDestruct();
