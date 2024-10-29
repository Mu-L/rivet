/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Build as actor_common$$build } from "../../common/types/Build";
import { actor } from "../../../../index";

export const GetBuildResponse: core.serialization.ObjectSchema<
    serializers.actor.GetBuildResponse.Raw,
    Rivet.actor.GetBuildResponse
> = core.serialization.object({
    build: actor_common$$build,
});

export declare namespace GetBuildResponse {
    interface Raw {
        build: actor.Build.Raw;
    }
}