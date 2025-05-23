/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const GlobalEventNotification: core.serialization.ObjectSchema<
    serializers.identity.GlobalEventNotification.Raw,
    Rivet.identity.GlobalEventNotification
> = core.serialization.object({
    title: core.serialization.string(),
    description: core.serialization.string(),
    thumbnailUrl: core.serialization.property("thumbnail_url", core.serialization.string()),
    url: core.serialization.string(),
});

export declare namespace GlobalEventNotification {
    export interface Raw {
        title: string;
        description: string;
        thumbnail_url: string;
        url: string;
    }
}
