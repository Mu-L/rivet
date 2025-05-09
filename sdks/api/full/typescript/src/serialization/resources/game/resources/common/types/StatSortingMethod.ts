/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";

export const StatSortingMethod: core.serialization.Schema<
    serializers.game.StatSortingMethod.Raw,
    Rivet.game.StatSortingMethod
> = core.serialization.enum_(["desc", "asc"]);

export declare namespace StatSortingMethod {
    export type Raw = "desc" | "asc";
}
