class Api {
    static #backend = null;
    #rpc

    /**
     * Create the api to interact with the backend
     * @type {(url: string) => Api}
     * @param url Url of the backend
     */
    constructor(url) {
        this.#rpc = simple_jsonrpc.connect_xhr(url);
    }

    /**
     * Get the api
     * @returns Api Return an instance of the api
     */
    static get backend() {
        if (this.#backend === null) this.#backend = new Api("http://127.0.0.1:3030");
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