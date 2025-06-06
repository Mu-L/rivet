/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../../../index";
import * as Rivet from "../../../../../../../../api/index";
import * as core from "../../../../../../../../core";

export const UpdateNamespaceCdnAuthUserRequest: core.serialization.ObjectSchema<
    serializers.cloud.games.namespaces.UpdateNamespaceCdnAuthUserRequest.Raw,
    Rivet.cloud.games.namespaces.UpdateNamespaceCdnAuthUserRequest
> = core.serialization.object({
    user: core.serialization.string(),
    password: core.serialization.string(),
});

export declare namespace UpdateNamespaceCdnAuthUserRequest {
    export interface Raw {
        user: string;
        password: string;
    }
}
