/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
import { common } from "../../../../index";
export declare const GetActorLogsResponse: core.serialization.ObjectSchema<serializers.actor.GetActorLogsResponse.Raw, Rivet.actor.GetActorLogsResponse>;
export declare namespace GetActorLogsResponse {
    interface Raw {
        lines: string[];
        timestamps: string[];
        watch: common.WatchResponse.Raw;
    }
}