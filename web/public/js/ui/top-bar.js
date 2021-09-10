class TopBar {
    /**
     *
     * @type {(element: HTMLElement, title: string, pagesConf: Array<any>) => TopBar}
     * @param element TopBar dom element
     * @param title Title of the page
     * @param pagesConf Pages configuration
     */
    constructor(element, title, pagesConf) {
        let thisPage = null;
        pagesConf.forEach(page => {
            if (document.location.pathname.indexOf(page.url) === 0 || (page.url.indexOf("/index.html") !== -1 &&
                document.location.pathname.indexOf(page.url.replace("/index.html", "/")) === 0))
                thisPage = page;
        });

        new Template("top-bar", {"page_title": title, "pages": pagesConf, "menu": thisPage && thisPage.menu}, element, () => {
            if (thisPage && thisPage.menu) {
                $('.ui.left.vertical.menu.sidebar').first()
                    .sidebar('attach events', '.sidebar.icon')
                    .sidebar('setting', 'transition', 'overlay')
                ;
            }
        });
    }
}

UiCore.registerTag("top-bar", element => {
    Config.loadConfig("pages", conf => {
        new TopBar(element, document.title, conf);
    });
});
