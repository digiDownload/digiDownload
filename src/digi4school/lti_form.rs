use crate::buffered_response::BufferedResponse;
use crate::try_expect;
use reqwest::{Client, Method, RequestBuilder, Url};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::str::FromStr;

const BAD_LTI_FORM_MESSAGE: &str = "Bad LTI-form: ";

/// LTI-form is just a bad redirect from digi4school
pub struct LTIForm {
    url: Url,
    method: Method,

    data: HashMap<String, String>,
}

// TODO document
// TODO remove all format! calls in expect
// TODO remove if let that will be called on each runtime invocation
macro_rules! form_attr_expect {
    ( $form:expr, $attr:expr, $alias:expr ) => {{
        let form = $form; // make sure to evaluate it
        let alias = if !$alias.is_empty() {
            format!(" ({})", $alias)
        } else {
            "".to_string()
        };

        form.attr($attr).expect(&*format!(
            "{BAD_LTI_FORM_MESSAGE}form input didn't specify '{}'{}.\nHTML Element: {}",
            $attr,
            alias,
            $form.html()
        ))
    }};
    ( $form:expr, $attr:expr ) => {
        form_attr_expect!($form, $attr, "")
    };
}

impl LTIForm {
    #[allow(clippy::question_mark)] // TODO remove. See https://github.com/rust-lang/rust-clippy/issues/12337
    pub fn new(raw_form: &BufferedResponse) -> Option<Self> {
        let doc = Html::parse_document(&raw_form.text());
        let selector = Selector::parse("form#lti").unwrap();

        let mut iter = doc.root_element().select(&selector);

        Some(try_expect!(Option<_>, "Invalid LTI-form", {
            let Some(html_form) = iter.next() else {
                return None;
            };

            assert!(
                iter.next().is_none(),
                "Found 2 valid HTML items for 'form#lti' css selector"
            );
            assert_eq!(
                form_attr_expect!(
                    html_form,
                    "enctype", // originally 'encType' but for some reason scraper converts it to lowercase
                    "encoding"
                ),
                "application/x-www-form-urlencoded",

                "{BAD_LTI_FORM_MESSAGE}Encoding Type is not 'application/x-www-form-urlencoded' but '{}'",
                html_form.attr("encType").unwrap()
            );
            assert_eq!(
                form_attr_expect!(html_form, "name"),
                "ltiLaunchForm",
                "Not an LTI-form despite having an #lti id"
            );

            LTIForm {
                url: Url::from_str(form_attr_expect!(html_form, "action", "url")).unwrap_or_else(
                    |_| panic!("{BAD_LTI_FORM_MESSAGE}Invalid 'action' (url) specified"),
                ),

                // TODO remove `.to_upppercase()` - See https://github.com/hyperium/http/issues/681
                method: Method::from_str(&form_attr_expect!(html_form, "method").to_uppercase())
                    .unwrap(), // unwrap is fine because this can't actually fail (bad FromStr implementation)

                data: html_form
                    .children()
                    .map(|node| {
                        let element = ElementRef::wrap(node).unwrap();

                        Some((
                            form_attr_expect!(element, "name").to_string(),
                            form_attr_expect!(element, "value").to_string(),
                        ))
                    })
                    .collect::<Option<HashMap<String, String>>>()
                    .unwrap(),
            }
        }))
    }

    pub async fn follow_recursively(
        self,
        client: &Client,
    ) -> Result<BufferedResponse, reqwest::Error> {
        let mut form = self;

        // loop because recursive async functions are cursed
        loop {
            let resp = BufferedResponse::new(form.build_request(client).send().await?).await?;

            form = match LTIForm::new(&resp) {
                Some(form) => form,
                None => {
                    return Ok(resp);
                }
            }
        }
    }

    pub fn build_request(self, client: &Client) -> RequestBuilder {
        client
            .request(self.method, self.url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(serde_urlencoded::to_string(self.data).unwrap())
    }
}
