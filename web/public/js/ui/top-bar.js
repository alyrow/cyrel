class TopBar {
    /**
     *
     * @type {(element: HTMLElement, title: string) => TopBar}
     * @param element TopBar dom element
     * @param title Title of the page
     */
    constructor(element, title) {
        new Template("top-bar", {"page_title": title}, element);
    }
}

UiCore.registerTag("top-bar", element => {
    new TopBar(element, document.title);
});
