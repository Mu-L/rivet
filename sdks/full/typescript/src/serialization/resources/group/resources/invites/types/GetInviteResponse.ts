/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Handle as group_common$$handle } from "../../common/types/Handle";
import { group } from "../../../../index";

export const GetInviteResponse: core.serialization.ObjectSchema<
    serializers.group.GetInviteResponse.Raw,
    Rivet.group.GetInviteResponse
> = core.serialization.object({
    group: group_common$$handle,
});

export declare namespace GetInviteResponse {
    interface Raw {
        group: group.Handle.Raw;
    }
}
