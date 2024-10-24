/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Build as actor_common$$build } from "../../common/types/Build";
import { actor } from "../../../../index";

export const ListBuildsResponse: core.serialization.ObjectSchema<
    serializers.actor.ListBuildsResponse.Raw,
    Rivet.actor.ListBuildsResponse
> = core.serialization.object({
    builds: core.serialization.list(actor_common$$build),
});

export declare namespace ListBuildsResponse {
    interface Raw {
        builds: actor.Build.Raw[];
    }
}
