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
}
