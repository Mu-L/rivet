/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { Timestamp } from "../../../../common/types/Timestamp";
import { DisplayName } from "../../../../common/types/DisplayName";
import { Config } from "./Config";

export const Full: core.serialization.ObjectSchema<serializers.cloud.version.Full.Raw, Rivet.cloud.version.Full> =
    core.serialization.object({
        versionId: core.serialization.property("version_id", core.serialization.string()),
        createTs: core.serialization.property("create_ts", Timestamp),
        displayName: core.serialization.property("display_name", DisplayName),
        config: Config,
    });

export declare namespace Full {
    export interface Raw {
        version_id: string;
        create_ts: Timestamp.Raw;
        display_name: DisplayName.Raw;
        config: Config.Raw;
    }
}
