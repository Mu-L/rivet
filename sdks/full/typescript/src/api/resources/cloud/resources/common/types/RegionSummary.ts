/**
 * This file was auto-generated by Fern from our API Definition.
 */

import * as Rivet from "../../../../../index";

/**
 * A region summary.
 */
export interface RegionSummary {
    regionId: string;
    /** A human readable short identifier used to references resources. Different than a `rivet.common#Uuid` because this is intended to be human readable. Different than `rivet.common#DisplayName` because this should not include special characters and be short. */
    regionNameId: string;
    /** The server provider of this region. */
    provider: string;
    /**
     * **Deprecated**
     * A universal region label given to this region.
     */
    universalRegion: Rivet.cloud.UniversalRegion;
    providerDisplayName: Rivet.DisplayName;
    regionDisplayName: Rivet.DisplayName;
}
