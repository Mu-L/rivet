/**
 * This file was auto-generated by Fern from our API Definition.
 */
import * as serializers from "../../../../../index";
import * as Rivet from "../../../../../../api/index";
import * as core from "../../../../../../core";
export declare const RegionTierExpenses: core.serialization.ObjectSchema<serializers.cloud.RegionTierExpenses.Raw, Rivet.cloud.RegionTierExpenses>;
export declare namespace RegionTierExpenses {
    interface Raw {
        namespace_id: string;
        region_id: string;
        tier_name_id: string;
        lobby_group_name_id: string;
        uptime: number;
        expenses: number;
    }
}
