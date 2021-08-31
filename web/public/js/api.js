class Api {
    static #backend = null;
    #rpc

    /**
     * Create the api to interact with the backend
     * @type {(url: string, fatalError: function) => Api}
     * @param url Url of the backend
     * @param fatalError Function to be called on fatal error
     */
    constructor(url, fatalError) {
        this.#rpc = simple_jsonrpc.connect_xhr(url, {
            onerror: fatalError
        });
    }

    /**
     * Return an instance of the api
     * @type {() => Api}
     */
    static get backend() {
        if (this.#backend === null) this.#backend = new Api("http://127.0.0.1:3030", e => {
            $('body')
                .toast({
                    class: 'error',
                    message: e
                })
            ;
        });
        return this.#backend;
    }

    /**
     * Function which login a user
     * @type {(username: string, password: string, onSuccess: function, onFailure: function) => void}
     * @param username Username
     * @param password Password
     * @param onSuccess When the server validate the login infos
     * @param onFailure When the server reject login infos
     */
    login(username, password, onSuccess, onFailure) {
        this.#rpc.call("login", [username, password])
            .then(res => onSuccess(res))
            .catch(err => onFailure(err));
    }
}
