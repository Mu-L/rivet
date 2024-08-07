/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { Summary as group_common$$summary } from "../resources/common/types/Summary";
import { WatchResponse as common$$watchResponse } from "../../common/types/WatchResponse";
import { group, common } from "../../index";

export const ListSuggestedResponse: core.serialization.ObjectSchema<
    serializers.group.ListSuggestedResponse.Raw,
    Rivet.group.ListSuggestedResponse
> = core.serialization.object({
    groups: core.serialization.list(group_common$$summary),
    watch: common$$watchResponse,
});

export declare namespace ListSuggestedResponse {
    interface Raw {
        groups: group.Summary.Raw[];
        watch: common.WatchResponse.Raw;
    }
}
