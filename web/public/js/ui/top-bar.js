class TopBar {
    /**
     * @type {(element: HTMLElement, title: string, pagesConf: Array<any>) => TopBar}
     * @param element TopBar dom element
     * @param title Title of the page
     * @param pagesConf Pages configuration
     */
    constructor(element, title, pagesConf) {
        let thisPage = null;
        var xDown = null;                                                        
        var yDown = null;

        pagesConf.forEach(page => {
            if (document.location.pathname.indexOf(page.url) === 0 || (page.url.indexOf("/index.html") !== -1 &&
                document.location.pathname.indexOf(page.url.replace("/index.html", "/")) === 0))
                thisPage = page;
            
            document.addEventListener('touchstart', handleTouchStart, false);
            document.addEventListener('touchmove', function (evt) {setTimeout(handleTouchMove, 500, evt)}, false);

            function getTouches(evt) {
                return evt.touches ||             // browser API
                        evt.originalEvent.touches; // jQuery
            }                                                     
                                                                                       
            function handleTouchStart(evt) {
                const firstTouch = getTouches(evt)[0];                                      
                xDown = firstTouch.clientX;                                      
                yDown = firstTouch.clientY;                                      
            };                                                
                                                                                       
            function handleTouchMove(evt) {
                if ( ! xDown || ! yDown ) {
                    return;
                }
                            
                var xUp = evt.touches[0].clientX;                                    
                var yUp = evt.touches[0].clientY;
            
                var xDiff = xDown - xUp;
                var yDiff = yDown - yUp;
                                                                                    
                if ( Math.abs( xDiff ) > Math.abs( yDiff ) ) {/*most significant*/
                        if ( xDiff < 0 ) { // swipe left
                            document.getElementsByClassName("sidebar icon").item(null).click()
                        }                       
                }

                xDown = null;
                yDown = null;                                             
            };
        });

        new Template("top-bar", {
            "page_title": title,
            "page_icon": thisPage.icon,
            "pages": pagesConf,
            "menu": thisPage && thisPage.menu,
            "logged": localStorage.getItem("__") !== null && localStorage.getItem("__") !== ""
        }, element, () => {
            if (thisPage && thisPage.menu) {
                $('.ui.left.vertical.menu.sidebar').first()
                    .sidebar('attach events', '.sidebar.icon', '.touchmove')
                    .sidebar('setting', 'transition', 'overlay')
                ;
            }
        });
    }

    static logout() {
        localStorage.removeItem("__");
        document.location.href = "/login.html";
    }
}

UiCore.registerTag("top-bar", element => {
    Config.loadConfig("pages", conf => {
        new TopBar(element, document.title, conf);
    });
});