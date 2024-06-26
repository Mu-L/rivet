/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as environments from "../../../../environments";
import * as core from "../../../../core";
import * as Rivet from "../../..";
import { Activities } from "../resources/activities/client/Client";
import { Events } from "../resources/events/client/Client";
import { Links } from "../resources/links/client/Client";
export declare namespace Identity {
    interface Options {
        environment?: core.Supplier<environments.RivetEnvironment | string>;
        token?: core.Supplier<core.BearerToken | undefined>;
        fetcher?: core.FetchFunction;
    }
    interface RequestOptions {
        timeoutInSeconds?: number;
        maxRetries?: number;
    }
}
export declare class Identity {
    protected readonly _options: Identity.Options;
    constructor(_options?: Identity.Options);
    /**
     * Gets or creates an identity.
     * Passing an existing identity token in the body refreshes the token.
     * Temporary Accounts
     * Until the identity is linked with the Rivet Hub (see `PrepareGameLink`), this identity will be temporary but still behave like all other identities.
     * This is intended to allow users to play the game without signing up while still having the benefits of having an account. When they are ready to save their account, they should be instructed to link their account (see `PrepareGameLink`).
     * Storing Token
     * `identity_token` should be stored in some form of persistent storage. The token should be read from storage and passed to `Setup` every time the client starts.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    setup(request?: Rivet.identity.SetupRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.SetupResponse>;
    /**
     * Fetches an identity profile.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    getProfile(identityId: string, request?: Rivet.identity.GetProfileRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.GetProfileResponse>;
    /**
     * Fetches the current identity's profile.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    getSelfProfile(request?: Rivet.identity.GetSelfProfileRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.GetProfileResponse>;
    /**
     * Fetches a list of identity handles.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    getHandles(request: Rivet.identity.GetHandlesRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.GetHandlesResponse>;
    /**
     * Fetches a list of identity summaries.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    getSummaries(request: Rivet.identity.GetSummariesRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.GetSummariesResponse>;
    /**
     * Updates profile of the current identity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    updateProfile(request?: Rivet.identity.UpdateProfileRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Validate contents of identity profile. Use to provide immediate feedback on profile changes before committing them.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    validateProfile(request?: Rivet.identity.ValidateProfileRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Fuzzy search for identities.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    search(request: Rivet.identity.SearchRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.SearchResponse>;
    /**
     * Sets the current identity's game activity. This activity will automatically be removed when the identity goes offline.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    setGameActivity(request: Rivet.identity.SetGameActivityRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Removes the current identity's game activity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    removeGameActivity(requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Updates the current identity's status.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    updateStatus(request: Rivet.identity.UpdateStatusRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Follows the given identity. In order for identities to be "friends", the other identity has to also follow this identity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    follow(identityId: string, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Unfollows the given identity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    unfollow(identityId: string, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Prepares an avatar image upload. Complete upload with `CompleteIdentityAvatarUpload`.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    prepareAvatarUpload(request: Rivet.identity.PrepareAvatarUploadRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.PrepareAvatarUploadResponse>;
    /**
     * Completes an avatar image upload. Must be called after the file upload process completes.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    completeAvatarUpload(uploadId: string, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Completes an avatar image upload. Must be called after the file upload process completes.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    signupForBeta(request: Rivet.identity.SignupForBetaRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * Creates an abuse report for an identity.
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    report(identityId: string, request?: Rivet.identity.ReportRequest, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    listFollowers(identityId: string, request?: Rivet.identity.ListFollowersRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.ListFollowersResponse>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    listFollowing(identityId: string, request?: Rivet.identity.ListFollowingRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.ListFollowingResponse>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    listFriends(request?: Rivet.identity.ListFriendsRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.ListFriendsResponse>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    listMutualFriends(identityId: string, request?: Rivet.identity.ListMutualFriendsRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.ListMutualFriendsResponse>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    listRecentFollowers(request?: Rivet.identity.ListRecentFollowersRequest, requestOptions?: Identity.RequestOptions): Promise<Rivet.identity.ListRecentFollowersResponse>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    ignoreRecentFollower(identityId: string, requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    markDeletion(requestOptions?: Identity.RequestOptions): Promise<void>;
    /**
     * @throws {@link Rivet.InternalError}
     * @throws {@link Rivet.RateLimitError}
     * @throws {@link Rivet.ForbiddenError}
     * @throws {@link Rivet.UnauthorizedError}
     * @throws {@link Rivet.NotFoundError}
     * @throws {@link Rivet.BadRequestError}
     */
    unmarkDeletion(requestOptions?: Identity.RequestOptions): Promise<void>;
    protected _activities: Activities | undefined;
    get activities(): Activities;
    protected _events: Events | undefined;
    get events(): Events;
    protected _links: Links | undefined;
    get links(): Links;
    protected _getAuthorizationHeader(): Promise<string | undefined>;
}
