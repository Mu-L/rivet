/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../../../../../../../environments";
import * as core from "../../../../../../../../../../core";
import * as Rivet from "../../../../../../../../../index";
export declare namespace Logs {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        fetcher?: core.FetchFunction;
    }
    interface RequestOptions {
        /** The maximum time to wait for a response in seconds. */
        timeoutInSeconds?: number;
        /** The number of times to retry the request. Defaults to 2. */
        maxRetries?: number;
        /** A hook to abort the request. */
        abortSignal?: AbortSignal;
    }
}
export declare class Logs {
    protected readonly _options: Logs.Options;
    constructor(_options?: Logs.Options);
    /**
     * Returns a list of lobbies for the given game namespace.
     *
     * @param {string} gameId
     * @param {string} namespaceId
     * @param {Rivet.cloud.games.namespaces.ListNamespaceLobbiesRequest} request
     * @param {Logs.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.cloud.games.namespaces.logs.listNamespaceLobbies("d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32", "d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32", {
     *         beforeCreateTs: new Date("2024-01-15T09:30:00.000Z")
     *     })
     */
    listNamespaceLobbies(gameId: string, namespaceId: string, request?: Rivet.cloud.games.namespaces.ListNamespaceLobbiesRequest, requestOptions?: Logs.RequestOptions): Promise<Rivet.cloud.games.namespaces.ListNamespaceLobbiesResponse>;
    /**
     * Returns a lobby from the given game namespace.
     *
     * @param {string} gameId
     * @param {string} namespaceId
     * @param {string} lobbyId
     * @param {Logs.RequestOptions} requestOptions - Request-specific configuration.
     *
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     *
     * @example
     *     await client.cloud.games.namespaces.logs.getNamespaceLobby("d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32", "d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32", "d5e9c84f-c2b2-4bf4-b4b0-7ffd7a9ffc32")
     */
    getNamespaceLobby(gameId: string, namespaceId: string, lobbyId: string, requestOptions?: Logs.RequestOptions): Promise<Rivet.cloud.games.namespaces.GetNamespaceLobbyResponse>;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}
