/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../index";
import * as Rivet from "../../../../api/index";
import * as core from "../../../../core";
import { Value as kv_common$$value } from "../resources/common/types/Value";
import { WatchResponse as common$$watchResponse } from "../../common/types/WatchResponse";
import { kv, common } from "../../index";

export const GetResponse: core.serialization.ObjectSchema<serializers.kv.GetResponse.Raw, Rivet.kv.GetResponse> =
    core.serialization.object({
        value: kv_common$$value,
        deleted: core.serialization.boolean().optional(),
        watch: common$$watchResponse,
    });

export declare namespace GetResponse {
    interface Raw {
        value?: kv.Value.Raw;
        deleted?: boolean | null;
        watch: common.WatchResponse.Raw;
    }
}
