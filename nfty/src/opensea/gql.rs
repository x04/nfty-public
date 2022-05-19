use crate::NftyConfig;
use net::{
    request::{CronetRequest, UploadData},
    CronetEngine, EngineParams,
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const EVENT_HISTORY_QUERY: &str = r#"query EventHistoryPollQuery(
  $archetype: ArchetypeInputType
  $categories: [CollectionSlug!]
  $chains: [ChainScalar!]
  $collections: [CollectionSlug!]
  $count: Int = 10
  $cursor: String
  $eventTimestamp_Gt: DateTime
  $eventTypes: [EventType!]
  $identity: IdentityInputType
  $showAll: Boolean = false
) {
  assetEvents(after: $cursor, archetype: $archetype, categories: $categories, chains: $chains, collections: $collections, eventTimestamp_Gt: $eventTimestamp_Gt, eventTypes: $eventTypes, first: $count, identity: $identity, includeHidden: true) {
    edges {
      node {
        assetBundle @include(if: $showAll) {
          relayId
          ...AssetCell_assetBundle
          ...bundle_url
          id
        }
        assetQuantity {
          asset @include(if: $showAll) {
            relayId
            assetContract {
              ...CollectionLink_assetContract
              id
            }
            ...AssetCell_asset
            ...asset_url
            collection {
              ...CollectionLink_collection
              id
            }
            id
          }
          ...quantity_data
          id
        }
        relayId
        eventTimestamp
        eventType
        customEventName
        offerExpired
        ...utilsAssetEventLabel
        devFee {
          asset {
            assetContract {
              chain
              id
            }
            id
          }
          quantity
          ...AssetQuantity_data
          id
        }
        devFeePaymentEvent {
          ...EventTimestamp_data
          id
        }
        fromAccount {
          address
          ...AccountLink_data
          id
        }
        price {
          quantity
          quantityInEth
          ...AssetQuantity_data
          id
        }
        endingPrice {
          quantity
          ...AssetQuantity_data
          id
        }
        seller {
          ...AccountLink_data
          id
        }
        toAccount {
          ...AccountLink_data
          id
        }
        winnerAccount {
          ...AccountLink_data
          id
        }
        ...EventTimestamp_data
        id
      }
    }
  }
}

fragment AccountLink_data on AccountType {
  address
  config
  isCompromised
  user {
    publicUsername
    id
  }
  displayName
  ...ProfileImage_data
  ...wallet_accountKey
  ...accounts_url
}

fragment AssetCell_asset on AssetType {
  collection {
    name
    id
  }
  name
  ...AssetMedia_asset
  ...asset_url
}

fragment AssetCell_assetBundle on AssetBundleType {
  assetQuantities(first: 2) {
    edges {
      node {
        asset {
          collection {
            name
            id
          }
          name
          ...AssetMedia_asset
          ...asset_url
          id
        }
        relayId
        id
      }
    }
  }
  name
  slug
}

fragment AssetMedia_asset on AssetType {
  animationUrl
  backgroundColor
  collection {
    displayData {
      cardDisplayStyle
    }
    id
  }
  isDelisted
  imageUrl
  displayImageUrl
}

fragment AssetQuantity_data on AssetQuantityType {
  asset {
    ...Price_data
    id
  }
  quantity
}

fragment CollectionLink_assetContract on AssetContractType {
  address
  blockExplorerLink
}

fragment CollectionLink_collection on CollectionType {
  name
  ...collection_url
  ...verification_data
}

fragment EventTimestamp_data on AssetEventType {
  eventTimestamp
  transaction {
    blockExplorerLink
    id
  }
}

fragment Price_data on AssetType {
  decimals
  imageUrl
  symbol
  usdSpotPrice
  assetContract {
    blockExplorerLink
    chain
    id
  }
}

fragment ProfileImage_data on AccountType {
  imageUrl
  address
}

fragment accounts_url on AccountType {
  address
  user {
    publicUsername
    id
  }
}

fragment asset_url on AssetType {
  assetContract {
    address
    chain
    id
  }
  tokenId
}

fragment bundle_url on AssetBundleType {
  slug
}

fragment collection_url on CollectionType {
  slug
}

fragment quantity_data on AssetQuantityType {
  asset {
    decimals
    id
  }
  quantity
}

fragment utilsAssetEventLabel on AssetEventType {
  isMint
  eventType
}

fragment verification_data on CollectionType {
  isMintable
  isSafelisted
  isVerified
}

fragment wallet_accountKey on AccountType {
  address
}
"#;

#[allow(dead_code)]
pub const EVENT_HISTORY_QUERY_OPTIMIZED: &str = r#"
query EventHistoryPollQuery(
  $collections: [CollectionSlug!]
  $count: Int = 10
  $cursor: String
  $eventTimestamp_Gt: DateTime
  $eventTypes: [EventType!]
  $identity: IdentityInputType
  $showTraits: Boolean = false
) {
  collections(first: 100, collections: $collections) @include(if: $showTraits) {
    edges {
      node {
        stringTraits {
          key
          counts {
            value
            count
          }
        }
      }
    }
  }
  assetEvents(after: $cursor, collections: $collections, eventTimestamp_Gt: $eventTimestamp_Gt, eventTypes: $eventTypes, first: $count, identity: $identity, includeHidden: true) {
    edges {
      node {
        assetQuantity {
          asset {
            name
            tokenId
            collection {
              slug
            }
            assetContract {
              address
            }
            traits(first: 100) {
              edges {
                node {
                  traitType
                  traitCount
                  value
                }
              }
            }
          }
          quantity
        }
        price {
  		  asset {
  			symbol
  		    assetContract {
  			  chain
  		    }
  		  }
  		  quantity
        }
        endingPrice {
  		  asset {
  		    symbol
  		      assetContract {
  			    chain
  			  }
  		  }
  		  quantity
        }
      }
    }
    pageInfo {
      endCursor
      hasNextPage
    }
  }
}
"#;

#[allow(dead_code)]
pub const SEARCH_QUERY: &str = r#"
  query AssetSearchQuery($chains: [ChainScalar!], $collection: CollectionSlug, $collections: [CollectionSlug!], $count: Int, $cursor: String, $numericTraits: [TraitRangeType!], $paymentAssets: [PaymentAssetSymbol!], $priceFilter: PriceFilterType, $query: String, $resultModel: SearchResultModel, $sortAscending: Boolean, $sortBy: SearchSortBy, $toggles: [SearchToggle!]) {
    query {
      # collection(collection: $collection) {
      #   numericTraits {
      #     key
      #     value {
      #       max
      #       min
      #     }
      #   }
      #   stringTraits {
      #     key
      #     counts {
      #       count
      #       value
      #     }
      #   }
      # }
      search(after: $cursor, chains: $chains, collections: $collections, first: $count, numericTraits: $numericTraits, paymentAssets: $paymentAssets, priceFilter: $priceFilter, querystring: $query, resultType: $resultModel, sortAscending: $sortAscending, sortBy: $sortBy, toggles: $toggles) {
        edges {
          node {
            asset {
              name
              tokenId
              assetContract {
                address
                chain
              }
              orderData {
                bestAsk {
                  orderType
                  quantity
                  decimals
                  paymentAssetQuantity {
                    quantity
                    asset {
                      decimals
                      symbol
                    }
                  }
                }
              }
              traits(first: 100) {
                edges {
                  node {
                    traitType
                    traitCount
                    value
                  }
                }
              }
            }
          }
        }
        totalCount
        pageInfo {
          endCursor
          hasNextPage
        }
      }
    }
  }
"#;

pub const _TRAIT_QUERY: &str = r#"
  query collectionQuery(
    $collection: CollectionSlug!
    $collections: [CollectionSlug!]
    $sortAscending: Boolean
    $sortBy: SearchSortBy
    $stringTraits: [TraitInputType!]
  ) {
    assets: query {
      collection(collection: $collection) {
      	stringTraits {
      	  key
      	  counts {
      	    count
      	    value
    	  }
      	}
      }
      search(collections: $collections, first: 100, querystring: "", resultType: ASSETS, sortAscending: $sortAscending, sortBy: $sortBy, stringTraits: $stringTraits) {
      	edges {
  	      node {
    		asset {
  	  		  name
      		  tokenId
      		  assetEventData {
        		lastSale {
          		  unitPriceQuantity {
            		asset {
      	  			  decimals
    		  		  symbol
    	  			}
    	  			quantity
          		  }
         		}
      		  }
      		  decimals
   	  		  orderData {
   	   			bestAsk {
        		  orderType
      	  		  quantity
    	    	  decimals
  	      	      paymentAssetQuantity {
        	  	    quantity
        	  	    asset {
      				  decimals
    				  symbol
    			    }
    			    quantity
    	          }
  	    		}
    	      }
    		}
      	  }
  	    }
  	    pageInfo {
  	      endCursor
  	      hasNextPage
  	    }
  	  }
    }
  }
"#;

#[allow(dead_code)]
pub const ORDERS_QUERY_OPTIMIZED: &str = r#"
  query OrdersQuery(
    $makerArchetype: ArchetypeInputType
    $first: Int = 10
    $after: String
  ) {
    orders(after: $after, first: $first, makerArchetype: $makerArchetype, isExpired: false, isValid: true, takerAssetIsPayment: true, sortAscending: true, sortBy: TAKER_ASSETS_USD_PRICE) {
      edges {
        node {
  		  id
          oldOrder
        }
      }
      pageInfo {
        endCursor
        hasNextPage
      }
    }
  }
"#;

pub const ORDERS_QUERY: &str = r#"query OrdersQuery(
  $cursor: String
  $count: Int = 10
  $excludeMaker: IdentityInputType
  $isExpired: Boolean
  $isValid: Boolean
  $isInactive: Boolean
  $maker: IdentityInputType
  $makerArchetype: ArchetypeInputType
  $makerAssetIsPayment: Boolean
  $takerArchetype: ArchetypeInputType
  $takerAssetCategories: [CollectionSlug!]
  $takerAssetCollections: [CollectionSlug!]
  $takerAssetIsOwnedBy: IdentityInputType
  $takerAssetIsPayment: Boolean
  $sortAscending: Boolean
  $sortBy: OrderSortOption
  $makerAssetBundle: BundleSlug
  $takerAssetBundle: BundleSlug
  $expandedMode: Boolean = false
  $isBid: Boolean = false
  $filterByOrderRules: Boolean = false
) {
  ...Orders_data_2gmi3R
}

fragment AccountLink_data on AccountType {
  address
  config
  isCompromised
  user {
    publicUsername
    id
  }
  displayName
  ...ProfileImage_data
  ...wallet_accountKey
  ...accounts_url
}

fragment AskPrice_data on OrderV2Type {
  dutchAuctionFinalPrice
  openedAt
  priceFnEndedAt
  makerAssetBundle {
    assetQuantities(first: 30) {
      edges {
        node {
          ...quantity_data
          id
        }
      }
    }
    id
  }
  takerAssetBundle {
    assetQuantities(first: 1) {
      edges {
        node {
          ...AssetQuantity_data
          id
        }
      }
    }
    id
  }
}

fragment AssetCell_assetBundle on AssetBundleType {
  assetQuantities(first: 2) {
    edges {
      node {
        asset {
          collection {
            name
            id
          }
          name
          ...AssetMedia_asset
          ...asset_url
          id
        }
        relayId
        id
      }
    }
  }
  name
  slug
}

fragment AssetMedia_asset on AssetType {
  animationUrl
  backgroundColor
  collection {
    displayData {
      cardDisplayStyle
    }
    id
  }
  isDelisted
  imageUrl
  displayImageUrl
}

fragment AssetQuantity_data on AssetQuantityType {
  asset {
    ...Price_data
    id
  }
  quantity
}

fragment CancelOrderButton_data on OrderV2Type {
  id
  item {
    __typename
    ... on AssetType {
      assetContract {
        address
        id
      }
      chain {
        identifier
      }
      collection {
        slug
        id
      }
    }
    ... on Node {
      __isNode: __typename
      id
    }
  }
  isFulfillable
  ...orderLink_data
  oldOrder
  orderType
  side
}

fragment Orders_data_2gmi3R on Query {
  orders(after: $cursor, excludeMaker: $excludeMaker, first: $count, isExpired: $isExpired, isInactive: $isInactive, isValid: $isValid, maker: $maker, makerArchetype: $makerArchetype, makerAssetIsPayment: $makerAssetIsPayment, takerArchetype: $takerArchetype, takerAssetCategories: $takerAssetCategories, takerAssetCollections: $takerAssetCollections, takerAssetIsOwnedBy: $takerAssetIsOwnedBy, takerAssetIsPayment: $takerAssetIsPayment, sortAscending: $sortAscending, sortBy: $sortBy, makerAssetBundle: $makerAssetBundle, takerAssetBundle: $takerAssetBundle, filterByOrderRules: $filterByOrderRules) {
    edges {
      node {
        closedAt
        isValid
        openedAt
        orderType
        maker {
          address
          ...AccountLink_data
          ...wallet_accountKey
          id
        }
        makerAsset: makerAssetBundle {
          assetQuantities(first: 1) {
            edges {
              node {
                asset {
                  assetContract {
                    address
                    chain
                    id
                  }
                  id
                }
                id
              }
            }
          }
          id
        }
        makerAssetBundle {
          assetQuantities(first: 30) {
            edges {
              node {
                ...AssetQuantity_data
                ...quantity_data
                id
              }
            }
          }
          id
        }
        relayId
        side
        taker {
          address
          ...AccountLink_data
          ...wallet_accountKey
          id
        }
        perUnitPrice {
          eth
        }
        price {
          usd
        }
        item @include(if: $isBid) {
          __typename
          ... on AssetType {
            collection {
              floorPrice
              id
            }
          }
          ... on AssetBundleType {
            collection {
              floorPrice
              id
            }
          }
          ... on Node {
            __isNode: __typename
            id
          }
        }
        takerAssetBundle {
          slug
          ...bundle_url
          assetQuantities(first: 1) {
            edges {
              node {
                asset {
                  ownedQuantity(identity: {})
                  decimals
                  symbol
                  relayId
                  assetContract {
                    address
                    id
                  }
                  ...asset_url
                  id
                }
                quantity
                ...AssetQuantity_data
                ...quantity_data
                id
              }
            }
          }
          id
        }
        ...AskPrice_data
        ...CancelOrderButton_data
        makerAssetBundleDisplay: makerAssetBundle @include(if: $expandedMode) {
          ...AssetCell_assetBundle
          id
        }
        takerAssetBundleDisplay: takerAssetBundle @include(if: $expandedMode) {
          ...AssetCell_assetBundle
          id
        }
        ...quantity_remaining
        id
        __typename
      }
      cursor
    }
    pageInfo {
      endCursor
      hasNextPage
    }
  }
}

fragment Price_data on AssetType {
  decimals
  imageUrl
  symbol
  usdSpotPrice
  assetContract {
    blockExplorerLink
    chain
    id
  }
}

fragment ProfileImage_data on AccountType {
  imageUrl
  address
}

fragment accounts_url on AccountType {
  address
  user {
    publicUsername
    id
  }
}

fragment asset_url on AssetType {
  assetContract {
    address
    chain
    id
  }
  tokenId
}

fragment bundle_url on AssetBundleType {
  slug
}

fragment orderLink_data on OrderV2Type {
  makerAssetBundle {
    assetQuantities(first: 30) {
      edges {
        node {
          asset {
            externalLink
            collection {
              externalUrl
              id
            }
            id
          }
          id
        }
      }
    }
    id
  }
}

fragment quantity_data on AssetQuantityType {
  asset {
    decimals
    id
  }
  quantity
}

fragment quantity_remaining on OrderV2Type {
  makerAsset: makerAssetBundle {
    assetQuantities(first: 1) {
      edges {
        node {
          asset {
            decimals
            id
          }
          quantity
          id
        }
      }
    }
    id
  }
  takerAsset: takerAssetBundle {
    assetQuantities(first: 1) {
      edges {
        node {
          asset {
            decimals
            id
          }
          quantity
          id
        }
      }
    }
    id
  }
  remainingQuantity
  side
}

fragment wallet_accountKey on AccountType {
  address
}
"#;

#[derive(Serialize, Deserialize)]
pub struct Query {
    id: String,
    query: String,
    variables: Value,
}

impl Query {
    pub fn new<S: ToString>(id: S, query: S, variables: Value) -> Self {
        Self {
            id: id.to_string(),
            query: query.to_string(),
            variables,
        }
    }
}

pub struct Executor {
    engine: CronetEngine,
    executor: net::Executor,
}

impl Executor {
    pub fn from_config(config: &NftyConfig) -> Result<Self, shared::Error> {
        // let client = config.create_http_client()?;

        let exec = net::Executor::new();
        let mut params = EngineParams::new();
        params.set_http2(true);
        params.set_brotli(true);
        params.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.0.4638.54 Safari/537.36");
        if let Some(proxy_url) = config.global.proxy_url.as_ref() {
            if !proxy_url.is_empty() {
                let url = Url::parse(proxy_url)?;
                if let Some(host) = url.host_str() {
                    if let Some(pass) = url.password() {
                        params.set_proxy_credentials(&format!(
                            "Basic {}",
                            base64::encode(format!("{}:{}", url.username(), pass))
                        ))
                    }
                    let proxy_uri = {
                        if let Some(port) = url.port() {
                            format!("{}://{}:{}", url.scheme(), host, port)
                        } else {
                            format!("{}://{}", url.scheme(), host)
                        }
                    };

                    params.set_proxy_uri(&proxy_uri);
                }
            }
        }
        let engine = CronetEngine::new(&mut params);

        Ok(Self::with_net(engine, exec))
    }

    pub fn with_net(engine: CronetEngine, executor: net::Executor) -> Self {
        Self { engine, executor }
    }

    pub async fn execute(&self, q: Query) -> Result<net::request::Response, crate::Error> {
        let signed_query = match q.id.as_str() {
            "OrderQuery" => "1c03875ecca199e4f680456647a41a6ba6d56dd0fc8a09811938f8077a2987d1",
            "OrdersQuery" => "0a5a5cff1edd7ed876f35bfec56f11b7532188c6e026cae2c0127dc455d2d250",
            "EventHistoryQuery" => {
                "2cac6c006160be8282dd4b8922506da9c751917bbfe83b6b961f37f1f6f632bb"
            }
            "EventHistoryPollQuery" => {
                "3a747c88c61dd95cfacf50a237d82179367c21b91058b9083692651e82f31145"
            }

            _ => unreachable!(format!("Invalid Query - {}", &q.id)),
        };

        let mut upload = UploadData::new(serde_json::to_vec(&q)?);

        let mut req = CronetRequest::new(&self.engine, &self.executor);
        req.set_method("POST");
        req.set_header("accept", "*/*");
        req.set_header("accept-encoding", "gzip, deflate, br");
        req.set_header("accept-language", "en-US;q=0.9");
        req.set_header("content-type", "application/json");
        req.set_header("cache-control", "no-cache");
        req.set_header("origin", "https://opensea.io");
        req.set_header("referer", "https://opensea.io/");
        req.set_header(
            "sec-ch-ua",
            "\"Google Chrome\";v=\"95\", \"Chromium\";v=\"95\", \";Not A Brand\";v=\"99\"",
        );
        req.set_header("sec-ch-ua-mobile", "?0");
        req.set_header("sec-ch-ua-platform", "\"Windows\"");
        req.set_header("sec-fetch-dest", "empty");
        req.set_header("sec-fetch-mode", "cors");
        req.set_header("sec-fetch-site", "same-site");
        req.set_header("x-api-key", "2f6f419a083c46de9d83ce3dbe7db601");
        req.set_header("x-build-id", "AbCgJ0LqCRNGkGdUglsVm");
        req.set_header("x-signed-query", signed_query);

        req.set_body(&mut upload);

        let resp = req.start("https://api.opensea.io/graphql/").await;

        if let Some(err) = resp.last_error {
            Err(err.as_str().into())
        } else {
            Ok(resp)
        }
    }
}
