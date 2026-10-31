# Decisions

> For the complete documentation index, see [llms.txt](/llms.txt). Markdown versions of documentation pages are available by appending `.md` to the page URL.

The Decisions API is now generally available. Review [what's new](#whats-new)
  since the public beta.

The Decisions API evaluates text, images, and JSON and returns typed answers about 10x faster than the Responses API. Get the probability that a condition is true, a choice from a fixed set, or a score against a rubric. Use those answers to classify content, route requests, and prioritize work in your application.

Try the Decisions API in the [Playground](https://platform.openai.com/decisions) to experiment with questions and inputs before writing code.

Send requests to `POST /v1/decisions` using `gpt-6-luna`, the only currently
  supported model.

## What's new

Since the public beta launched, the Decisions API has added these capabilities:

- [**Prompt caching.**](#reuse-shared-input-with-prompt-caching) Reuse shared input across requests and receive cache discounts.
- [**Images in question instructions.**](#include-images-in-question-instructions) Give each question its own images to compare against shared evidence.
- [**JSON input.**](#evaluate-json-data) Pass structured data directly with an `input_json` content part.
- [**More questions per request.**](#ask-multiple-questions) Ask up to 400 questions per request, up from 200 during the public beta.
- [**Hosted image inputs.**](#check-an-image-for-visible-damage) Pass publicly accessible HTTP or HTTPS image URLs as well as inline base64 data URLs.
- [**Conversation history.**](#evaluate-a-conversation) Evaluate messages alongside function calls, custom tool calls, and their outputs.
- [**Organization usage reporting.**](#monitor-usage) Track token usage and request counts.
- [**Native GPT-Live integration.**](https://developers.openai.com/api/docs/guides/decisions-voice) Let Live call Decisions directly, or trigger decisions as input or output transcripts arrive.
- [**Private Link.**](https://developers.openai.com/api/docs/guides/private-link) Send Decisions requests over a private connection.

## How decisions work

A request has three parts:

| Field       | Purpose                                                                                                         |
| ----------- | --------------------------------------------------------------------------------------------------------------- |
| `model`     | The model that evaluates the request. Currently, only `gpt-6-luna` is supported.                                |
| `input`     | Shared evidence for the questions: a text string or an ordered array of messages, tool calls, and tool outputs. |
| `questions` | What to evaluate, including each question's type, instructions, and any allowed choices or score levels.        |

The response contains an `answers` array. Give each question a unique `name` to identify its answer; the API echoes that name in the response.

### Choose a question type

| Type        | Use it to                                                       | Main result                                                        |
| ----------- | --------------------------------------------------------------- | ------------------------------------------------------------------ |
| `predicate` | Check a condition, such as visible damage or passage relevance. | `probability`: an estimate from 0 to 1 that the condition is true. |
| `choice`    | Select one option, such as a department or content category.    | `choice`: one of your supplied values.                             |
| `score`     | Rate an input against ordered levels, such as issue severity.   | `score`: the probability-weighted average of the level indices.    |

Both `choice` and `score` return probabilities over discrete options. Use `choice` for categories without an order, such as departments. Use `score` for ordered levels, such as severity; it takes the probability-weighted average of their numeric indices to produce a score that can fall between levels.

Use Decisions when your application needs one of these answer types. Use [Structured Outputs](https://developers.openai.com/api/docs/guides/structured-outputs) with the Responses API when you need to generate an object that follows your own JSON schema, such as extracted fields or a written explanation, or [function calling](https://developers.openai.com/api/docs/guides/function-calling) when you need a model to request a tool call with arguments.

### Evaluate a conversation

Pass conversation history in `input` to evaluate a request in context. Include messages and any function or custom tool calls and outputs in order. Match each tool output to its call with `call_id`.

The supported content depends on the message role or item type:

| Input item                     | Supported content                                                  |
| ------------------------------ | ------------------------------------------------------------------ |
| User message                   | Text, images, and JSON                                             |
| Developer or system message    | Input text only; system messages are treated as developer messages |
| Assistant message              | Output text or refusals                                            |
| Function or custom tool output | Text and images, associated with a matching tool call              |

Tool calls and outputs provide evidence for your questions. Decisions returns typed answers; your application runs any actions those answers select. Files, audio, and item references aren't supported. See the [create decision reference](https://developers.openai.com/api/reference/resources/decisions/methods/create) for accepted input fields.

## Evaluate JSON data

Use an `input_json` content part to pass structured evidence without first converting it to a string. Put the JSON object or array in `data`, alongside any text or image parts in a user message. Within message content, `input_json` is supported only for the `user` role.

For example, send this JSON request body to `POST /v1/decisions` to evaluate a support ticket against an order record:

```json
{
  "model": "gpt-6-luna",
  "input": [
    {
      "role": "user",
      "content": [
        {
          "type": "input_text",
          "text": "Evaluate the ticket using the order evidence."
        },
        {
          "type": "input_json",
          "data": {
            "ticket": { "body": "Please refund the duplicate charge." },
            "order": { "charges": [49, 49] }
          }
        }
      ]
    }
  ],
  "questions": [
    {
      "type": "predicate",
      "name": "duplicate_charge",
      "instructions": "Does the ticket describe a duplicate charge supported by the order?"
    }
  ]
}
```

JSON keys such as `role` or `image_url` inside `data` are ordinary data: they don't create messages or fetch images. The answer still uses the selected question type; JSON input doesn't change the response format.

## Check an image for visible damage

Use a `predicate` question to check a product photo for visible damage. This request combines the image with instructions to look for a crack, tear, or dent.



Check an image for visible damage

```bash
IMAGE_BASE64="$(base64 < product.png | tr -d '\r\n')"

curl https://api.openai.com/v1/decisions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  --data-binary @- <<JSON
{
  "model": "gpt-6-luna",
  "input": [{
    "role": "user",
    "content": [
      {"type": "input_text", "text": "Inspect the product in this photo."},
      {"type": "input_image", "image_url": "data:image/png;base64,$IMAGE_BASE64"}
    ]
  }],
  "questions": [{
    "type": "predicate",
    "name": "visible_damage",
    "instructions": "Does the product have visible damage, such as a crack, tear, or dent? Ignore shadows and damage to the packaging."
  }]
}
JSON
```

```javascript
import { readFile } from "node:fs/promises";
import OpenAI from "openai";

const client = new OpenAI();
const imageBase64 = (await readFile("product.png")).toString("base64");
const decision = await client.decisions.create({
  model: "gpt-6-luna",
  input: [
    {
      role: "user",
      content: [
        { type: "input_text", text: "Inspect the product in this photo." },
        {
          type: "input_image",
          image_url: `data:image/png;base64,${imageBase64}`,
        },
      ],
    },
  ],
  questions: [
    {
      type: "predicate",
      name: "visible_damage",
      instructions:
        "Does the product have visible damage, such as a crack, tear, or dent? Ignore shadows and damage to the packaging.",
    },
  ],
});

const answer = decision.answers[0];
if (answer.type === "refusal") {
  console.log(`Refused: ${answer.name}`);
} else if (answer.type === "predicate") {
  console.log(`Visible damage probability: ${answer.probability}`);
}
```

```python
import base64
from pathlib import Path

from openai import OpenAI

client = OpenAI()
image_base64 = base64.b64encode(Path("product.png").read_bytes()).decode("ascii")

decision = client.decisions.create(
    model="gpt-6-luna",
    input=[
        {
            "role": "user",
            "content": [
                {"type": "input_text", "text": "Inspect the product in this photo."},
                {
                    "type": "input_image",
                    "image_url": f"data:image/png;base64,{image_base64}",
                },
            ],
        }
    ],
    questions=[
        {
            "type": "predicate",
            "name": "visible_damage",
            "instructions": (
                "Does the product have visible damage, such as a crack, tear, or dent? "
                "Ignore shadows and damage to the packaging."
            ),
        }
    ],
)

answer = decision.answers[0]
if answer.type == "refusal":
    print(f"Refused: {answer.name}")
elif answer.type == "predicate":
    print(f"Visible damage probability: {answer.probability}")
```

```go
package main

import (
	"context"
	"encoding/base64"
	"fmt"
	"os"

	"github.com/openai/openai-go/v3"
)

func main() {
	image, err := os.ReadFile("product.png")
	if err != nil {
		panic(err)
	}
	client := openai.NewClient()
	decision, err := client.Decisions.New(context.Background(), openai.DecisionNewParams{
		Model: "gpt-6-luna",
		Input: openai.DecisionNewParamsInputUnion{
			OfDecisionInputMessageArray: []openai.DecisionInputMessageParam{{
				Content: openai.DecisionInputMessageContentUnionParam{
					OfParts: []openai.DecisionInputPartUnionParam{
						{OfInputText: &openai.DecisionInputTextParam{Text: "Inspect the product in this photo."}},
						{OfInputImage: &openai.DecisionInputImageParam{
							ImageURL: "data:image/png;base64," + base64.StdEncoding.EncodeToString(image),
						}},
					},
				},
			}},
		},
		Questions: []openai.DecisionNewParamsQuestionUnion{{
			OfPredicate: &openai.DecisionNewParamsQuestionPredicate{
				Name:         openai.String("visible_damage"),
				Instructions: "Does the product have visible damage, such as a crack, tear, or dent? Ignore shadows and damage to the packaging.",
			},
		}},
	})
	if err != nil {
		panic(err)
	}
	switch answer := decision.Answers[0].AsAny().(type) {
	case openai.DecisionAnswerPredicate:
		fmt.Println(answer.Probability)
	case openai.DecisionAnswerRefusal:
		fmt.Printf("Decision refused for %s\n", answer.Name)
	default:
		panic("unexpected answer type")
	}
}
```

```java
import com.openai.models.decisions.DecisionCreateParams;
import com.openai.models.decisions.DecisionInputImage;
import com.openai.models.decisions.DecisionInputMessage;
import com.openai.models.decisions.DecisionInputPart;
import com.openai.models.decisions.DecisionInputText;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Base64;
import java.util.List;

String imageBase64 =
    Base64.getEncoder().encodeToString(Files.readAllBytes(Path.of("product.png")));
var message =
    DecisionInputMessage.builder()
        .contentOfParts(
            List.of(
                DecisionInputPart.ofInputText(
                    DecisionInputText.builder()
                        .text("Inspect the product in this photo.")
                        .build()),
                DecisionInputPart.ofInputImage(
                    DecisionInputImage.builder()
                        .imageUrl("data:image/png;base64," + imageBase64)
                        .build())))
        .build();
var decision =
    client
        .decisions()
        .create(
            DecisionCreateParams.builder()
                .model("gpt-6-luna")
                .inputOfDecisionInputMessages(List.of(message))
                .addQuestion(
                    DecisionCreateParams.Question.Predicate.builder()
                        .name("visible_damage")
                        .instructions(
                            "Does the product have visible damage, such as a crack, tear, or"
                                + " dent? Ignore shadows and damage to the packaging.")
                        .build())
                .build());

var answer = decision.answers().get(0);
if (answer.isRefusal()) {
  System.out.println("Refused: " + answer.asRefusal().name().orElse("visible_damage"));
} else {
  System.out.println(answer.asPredicate().probability());
}
```

```ruby
require "base64"
require "openai"

image_base64 = Base64.strict_encode64(File.binread("product.png"))
client = OpenAI::Client.new

decision = client.decisions.create(
  model: "gpt-6-luna",
  input: [
    {
      role: :user,
      content: [
        {
          type: :input_text,
          text: "Inspect the product in this photo."
        },
        {
          type: :input_image,
          image_url: "data:image/png;base64,#{image_base64}"
        }
      ]
    }
  ],
  questions: [
    {
      type: :predicate,
      name: "visible_damage",
      instructions: "Does the product have visible damage, such as a crack, tear, or dent? Ignore shadows and damage to the packaging."
    }
  ]
)

answer = decision.answers.fetch(0)
case answer
when OpenAI::Models::Decision::Answer::Predicate
  puts(answer.probability)
when OpenAI::Models::Decision::Answer::Refusal
  warn("Decision refused for #{answer.name}")
else
  raise("Unexpected answer type: #{answer.type}")
end
```


An illustrative response excerpt:

```json
{
  "answers": [
    {
      "type": "predicate",
      "name": "visible_damage",
      "probability": 0.92
    }
  ]
}
```

The `probability` is the model's estimate that the condition is true. Use it to flag photos for review based on a threshold you choose.

Provide images as inline base64 data URLs or publicly accessible HTTP or HTTPS URLs. This endpoint doesn't support `file_id` inputs. Combine `input_text` and `input_image` parts in a user message to evaluate images together with instructions or other context.

## Include images in question instructions

Put evidence that all questions share in `input`. To give each question a different image, set its `instructions` to a nonempty array of `input_text` and `input_image` parts. This works for predicate, choice, and score questions. Text-only instructions can remain a string.

For example, send this JSON request body to `POST /v1/decisions` to compare a candidate against a shared reference. Replace the two example image URLs with publicly accessible images, or use base64 image data URLs:

```json
{
  "model": "gpt-6-luna",
  "input": [
    {
      "role": "user",
      "content": [
        {
          "type": "input_text",
          "text": "The shared image is reference image A."
        },
        {
          "type": "input_image",
          "image_url": "https://example.com/reference.jpg"
        }
      ]
    }
  ],
  "questions": [
    {
      "type": "predicate",
      "name": "candidate_matches_reference",
      "instructions": [
        {
          "type": "input_text",
          "text": "Is candidate image B a derivative or variant of reference image A, including recompression or resizing?"
        },
        {
          "type": "input_image",
          "image_url": "https://example.com/candidate.jpg"
        }
      ]
    }
  ]
}
```

When adding images to question instructions:

- Add another question for each candidate image. Each question evaluates its own candidate against the same shared reference.
- Supply content parts directly in `instructions`, without a message wrapper or `role`.
- Question images share the request's image limits with images in `input`.

## Select from fixed options

A `choice` question selects one value from the options you provide. Use distinct values and descriptions that explain when each option applies.



This request routes a customer complaint:

Route a customer complaint

```bash
curl https://api.openai.com/v1/decisions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-6-luna",
    "input": "I was charged twice for my order.",
    "questions": [{
      "type": "choice",
      "name": "department",
      "instructions": "Which department should handle this complaint?",
      "choices": [
        {"value": "billing", "description": "Payments, invoices, and refunds."},
        {"value": "technical", "description": "Problems using the product."},
        {"value": "shipping", "description": "Delivery and tracking."},
        {"value": "other", "description": "Requests outside these categories."}
      ]
    }]
  }'
```

```javascript
import OpenAI from "openai";

const client = new OpenAI();
const decision = await client.decisions.create({
  model: "gpt-6-luna",
  input: "I was charged twice for my order.",
  questions: [
    {
      type: "choice",
      name: "department",
      instructions: "Which department should handle this complaint?",
      choices: [
        { value: "billing", description: "Payments, invoices, and refunds." },
        { value: "technical", description: "Problems using the product." },
        { value: "shipping", description: "Delivery and tracking." },
        { value: "other", description: "Requests outside these categories." },
      ],
    },
  ],
});

const answer = decision.answers[0];
if (answer.type === "refusal") {
  console.log(`Refused: ${answer.name}`);
} else if (answer.type === "choice") {
  console.log(
    `Department: ${answer.choice} (confidence: ${answer.confidence})`
  );
}
```

```python
from openai import OpenAI

client = OpenAI()
decision = client.decisions.create(
    model="gpt-6-luna",
    input="I was charged twice for my order.",
    questions=[
        {
            "type": "choice",
            "name": "department",
            "instructions": "Which department should handle this complaint?",
            "choices": [
                {"value": "billing", "description": "Payments, invoices, and refunds."},
                {"value": "technical", "description": "Problems using the product."},
                {"value": "shipping", "description": "Delivery and tracking."},
                {"value": "other", "description": "Requests outside these categories."},
            ],
        }
    ],
)

answer = decision.answers[0]
if answer.type == "refusal":
    print(f"Refused: {answer.name}")
elif answer.type == "choice":
    print(f"Department: {answer.choice} (confidence: {answer.confidence})")
```

```go
package main

import (
	"context"
	"fmt"

	"github.com/openai/openai-go/v3"
)

func main() {
	client := openai.NewClient()
	decision, err := client.Decisions.New(context.Background(), openai.DecisionNewParams{
		Model: "gpt-6-luna",
		Input: openai.DecisionNewParamsInputUnion{OfString: openai.String("I was charged twice for my order.")},
		Questions: []openai.DecisionNewParamsQuestionUnion{{
			OfChoice: &openai.DecisionNewParamsQuestionChoice{
				Name:         openai.String("department"),
				Instructions: "Which department should handle this complaint?",
				Choices: []openai.DecisionNewParamsQuestionChoiceChoice{
					{
						Value:       openai.DecisionNewParamsQuestionChoiceChoiceValueUnion{OfString: openai.String("billing")},
						Description: openai.String("Payments, invoices, and refunds."),
					},
					{
						Value:       openai.DecisionNewParamsQuestionChoiceChoiceValueUnion{OfString: openai.String("technical")},
						Description: openai.String("Problems using the product."),
					},
					{
						Value:       openai.DecisionNewParamsQuestionChoiceChoiceValueUnion{OfString: openai.String("shipping")},
						Description: openai.String("Delivery and tracking."),
					},
					{
						Value:       openai.DecisionNewParamsQuestionChoiceChoiceValueUnion{OfString: openai.String("other")},
						Description: openai.String("Requests outside these categories."),
					},
				},
			},
		}},
	})
	if err != nil {
		panic(err)
	}
	switch answer := decision.Answers[0].AsAny().(type) {
	case openai.DecisionAnswerChoice:
		fmt.Println(answer.Choice.AsString(), answer.Confidence, answer.Probabilities)
	case openai.DecisionAnswerRefusal:
		fmt.Printf("Decision refused for %s\n", answer.Name)
	default:
		panic("unexpected answer type")
	}
}
```

```java
import com.openai.models.decisions.DecisionChoiceOption;
import com.openai.models.decisions.DecisionCreateParams;
import com.openai.models.decisions.DecisionCreateParams.Question.Choice;

var decision =
    client
        .decisions()
        .create(
            DecisionCreateParams.builder()
                .model("gpt-6-luna")
                .input("I was charged twice for my order.")
                .addQuestion(
                    Choice.builder()
                        .name("department")
                        .instructions("Which department should handle this complaint?")
                        .addChoice(
                            DecisionChoiceOption.builder()
                                .value("billing")
                                .description("Payments, invoices, and refunds.")
                                .build())
                        .addChoice(
                            DecisionChoiceOption.builder()
                                .value("technical")
                                .description("Problems using the product.")
                                .build())
                        .addChoice(
                            DecisionChoiceOption.builder()
                                .value("shipping")
                                .description("Delivery and tracking.")
                                .build())
                        .addChoice(
                            DecisionChoiceOption.builder()
                                .value("other")
                                .description("Requests outside these categories.")
                                .build())
                        .build())
                .build());

var answer = decision.answers().get(0);
if (answer.isRefusal()) {
  System.out.println("Refused: " + answer.asRefusal().name().orElse("department"));
} else {
  var choice = answer.asChoice();
  System.out.println(choice.choice().asString());
  System.out.println(choice.probabilities());
  System.out.println(choice.confidence());
}
```

```ruby
require "openai"

client = OpenAI::Client.new
decision = client.decisions.create(
  model: "gpt-6-luna",
  input: "I was charged twice for my order.",
  questions: [
    {
      type: :choice,
      name: "department",
      instructions: "Which department should handle this complaint?",
      choices: [
        {
          value: "billing",
          description: "Payments, invoices, and refunds."
        },
        {
          value: "technical",
          description: "Problems using the product."
        },
        {
          value: "shipping",
          description: "Delivery and tracking."
        },
        {
          value: "other",
          description: "Requests outside these categories."
        }
      ]
    }
  ]
)

answer = decision.answers.fetch(0)
case answer
when OpenAI::Models::Decision::Answer::Choice
  puts(answer.choice, answer.confidence, answer.probabilities)
when OpenAI::Models::Decision::Answer::Refusal
  warn("Decision refused for #{answer.name}")
else
  raise("Unexpected answer type: #{answer.type}")
end
```


An illustrative response excerpt:

```json
{
  "answers": [
    {
      "type": "choice",
      "name": "department",
      "choice": "billing",
      "probabilities": [
        { "value": "billing", "probability": 0.95 },
        { "value": "technical", "probability": 0.02 },
        { "value": "shipping", "probability": 0.01 },
        { "value": "other", "probability": 0.02 }
      ],
      "confidence": 0.93
    }
  ]
}
```

The answer's `choice` field contains a supplied value, here `"billing"`. It also includes a `probabilities` array for the options and a `confidence` field. See [Interpret the answers](#interpret-the-answers) for guidance on setting thresholds.

Include a fallback option such as `"other"` when your categories don't cover every possible input. Your application can send that result to a general review queue.

## Score against a rubric

A `score` question evaluates an input against 2–10 ordered `levels`. Define the criteria for each level and arrange them from lowest to highest.



Score an issue against severity levels

```bash
curl https://api.openai.com/v1/decisions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-6-luna",
    "input": "Export fails in Safari but works in Chrome.",
    "questions": [{
      "type": "score",
      "name": "severity",
      "instructions": "How severe is this issue?",
      "levels": [
        {"label": "Cosmetic", "description": "Appearance only; no lost functionality."},
        {"label": "Workaround available", "description": "A task fails, but another way works."},
        {"label": "Fully blocked", "description": "A task fails with no workaround."}
      ]
    }]
  }'
```

```javascript
import OpenAI from "openai";

const client = new OpenAI();
const decision = await client.decisions.create({
  model: "gpt-6-luna",
  input: "Export fails in Safari but works in Chrome.",
  questions: [
    {
      type: "score",
      name: "severity",
      instructions: "How severe is this issue?",
      levels: [
        {
          label: "Cosmetic",
          description: "Appearance only; no lost functionality.",
        },
        {
          label: "Workaround available",
          description: "A task fails, but another way works.",
        },
        {
          label: "Fully blocked",
          description: "A task fails with no workaround.",
        },
      ],
    },
  ],
});

const answer = decision.answers[0];
if (answer.type === "refusal") {
  console.log(`Refused: ${answer.name}`);
} else if (answer.type === "score") {
  console.log(`Severity: ${answer.score} (confidence: ${answer.confidence})`);
}
```

```python
from openai import OpenAI

client = OpenAI()
decision = client.decisions.create(
    model="gpt-6-luna",
    input="Export fails in Safari but works in Chrome.",
    questions=[
        {
            "type": "score",
            "name": "severity",
            "instructions": "How severe is this issue?",
            "levels": [
                {
                    "label": "Cosmetic",
                    "description": "Appearance only; no lost functionality.",
                },
                {
                    "label": "Workaround available",
                    "description": "A task fails, but another way works.",
                },
                {
                    "label": "Fully blocked",
                    "description": "A task fails with no workaround.",
                },
            ],
        }
    ],
)

answer = decision.answers[0]
if answer.type == "refusal":
    print(f"Refused: {answer.name}")
elif answer.type == "score":
    print(f"Severity: {answer.score} (confidence: {answer.confidence})")
```

```go
package main

import (
	"context"
	"fmt"

	"github.com/openai/openai-go/v3"
)

func main() {
	client := openai.NewClient()
	decision, err := client.Decisions.New(context.Background(), openai.DecisionNewParams{
		Model: "gpt-6-luna",
		Input: openai.DecisionNewParamsInputUnion{OfString: openai.String("Export fails in Safari but works in Chrome.")},
		Questions: []openai.DecisionNewParamsQuestionUnion{{
			OfScore: &openai.DecisionNewParamsQuestionScore{
				Name:         openai.String("severity"),
				Instructions: "How severe is this issue?",
				Levels: []openai.DecisionNewParamsQuestionScoreLevel{
					{Label: "Cosmetic", Description: openai.String("Appearance only; no lost functionality.")},
					{Label: "Workaround available", Description: openai.String("A task fails, but another way works.")},
					{Label: "Fully blocked", Description: openai.String("A task fails with no workaround.")},
				},
			},
		}},
	})
	if err != nil {
		panic(err)
	}
	switch answer := decision.Answers[0].AsAny().(type) {
	case openai.DecisionAnswerScore:
		fmt.Println(answer.Score, answer.Confidence, answer.Probabilities)
	case openai.DecisionAnswerRefusal:
		fmt.Printf("Decision refused for %s\n", answer.Name)
	default:
		panic("unexpected answer type")
	}
}
```

```java
import com.openai.models.decisions.DecisionCreateParams;
import com.openai.models.decisions.DecisionCreateParams.Question.Score;

var decision =
    client
        .decisions()
        .create(
            DecisionCreateParams.builder()
                .model("gpt-6-luna")
                .input("Export fails in Safari but works in Chrome.")
                .addQuestion(
                    Score.builder()
                        .name("severity")
                        .instructions("How severe is this issue?")
                        .addLevel(
                            Score.Level.builder()
                                .label("Cosmetic")
                                .description("Appearance only; no lost functionality.")
                                .build())
                        .addLevel(
                            Score.Level.builder()
                                .label("Workaround available")
                                .description("A task fails, but another way works.")
                                .build())
                        .addLevel(
                            Score.Level.builder()
                                .label("Fully blocked")
                                .description("A task fails with no workaround.")
                                .build())
                        .build())
                .build());

var answer = decision.answers().get(0);
if (answer.isRefusal()) {
  System.out.println("Refused: " + answer.asRefusal().name().orElse("severity"));
} else {
  var score = answer.asScore();
  System.out.println(score.score());
  System.out.println(score.probabilities());
  System.out.println(score.confidence());
}
```

```ruby
require "openai"

client = OpenAI::Client.new
decision = client.decisions.create(
  model: "gpt-6-luna",
  input: "Export fails in Safari but works in Chrome.",
  questions: [
    {
      type: :score,
      name: "severity",
      instructions: "How severe is this issue?",
      levels: [
        {
          label: "Cosmetic",
          description: "Appearance only; no lost functionality."
        },
        {
          label: "Workaround available",
          description: "A task fails, but another way works."
        },
        {
          label: "Fully blocked",
          description: "A task fails with no workaround."
        }
      ]
    }
  ]
)

answer = decision.answers.fetch(0)
case answer
when OpenAI::Models::Decision::Answer::Score
  puts(answer.score, answer.confidence, answer.probabilities)
when OpenAI::Models::Decision::Answer::Refusal
  warn("Decision refused for #{answer.name}")
else
  raise("Unexpected answer type: #{answer.type}")
end
```


An illustrative response excerpt:

```json
{
  "answers": [
    {
      "type": "score",
      "name": "severity",
      "score": 1.1,
      "probabilities": [
        { "value": 0, "label": "Cosmetic", "probability": 0.1 },
        { "value": 1, "label": "Workaround available", "probability": 0.7 },
        { "value": 2, "label": "Fully blocked", "probability": 0.2 }
      ],
      "confidence": 0.55
    }
  ]
}
```

Level indices start at 0. Here, 0 means cosmetic, 1 means a workaround is available, and 2 means fully blocked. The returned `score` is a probability-weighted average, so it can fall between levels. In this example, probabilities of 0.1, 0.7, and 0.2 produce a score of 1.1.

The answer also includes `confidence` and the per-level `probabilities`. The score summarizes the distribution across levels. Use `choice` to select a single category.

## Ask multiple questions

Include between 1 and 400 questions in a request. Put independent questions in the same `questions` array to evaluate shared input. For a product photo, one request can check for damage and classify the product category. Each question can use a different type.

For decisions that depend on an earlier answer, send separate requests. For example, check for damage first, then use the result to decide whether to request a repair category.

Write questions around observable criteria. Separate different concerns into different questions, give choices distinct meanings, and define score levels so that adjacent levels have distinct criteria.

## Interpret the answers

Predicates return the estimated probability that a condition is true. Choice and score answers return a probability distribution and a separate `confidence` field.

Use labeled examples from your application to set thresholds for routing, filtering, or review. Choose thresholds based on the cost of false positives and false negatives.

## Reuse shared input with prompt caching

Prompt caching lets repeated requests reuse a matching prefix of shared `input` and receive discounted cache-read pricing. For example, keep a reference document or image in `input` and change the questions for each request. Content in `questions`, including question-specific images, is outside the reusable prefix.

By default, Decisions places an implicit cache breakpoint at the end of shared `input`. Existing requests can benefit without changing their request body. The reusable prefix must contain at least 1,024 tokens; question content doesn't count toward this threshold. Keep the prefix identical across requests to allow reuse.

Use `prompt_cache_key` to partition cache reuse by a caller-provided key. For control over the boundary, add `prompt_cache_breakpoint: {"mode": "explicit"}` to text or image content parts in shared `input`. Set `prompt_cache_options.mode` to `"explicit"` to use only marked boundaries. In explicit mode, a request with no marked boundaries doesn't use prompt caching. Breakpoints aren't supported in question instructions.

Check `usage.input_tokens_details.cached_tokens` for cache reads and `usage.input_tokens_details.cache_write_tokens` for cache writes. Both counts are included in `usage.input_tokens`; don't add them to that total. See the [prompt caching guide](https://developers.openai.com/api/docs/guides/prompt-caching) for caching concepts and the [Decisions reference](https://developers.openai.com/api/reference/resources/decisions/methods/create) for supported request controls.

## Monitor usage

Use `GET /v1/organization/usage/decisions` to retrieve your organization's Decisions token usage and request counts. Authenticate with an admin API key and the `api.usage.read` permission.

Choose a time range and bucket width, then filter or group the results by project, API key, or model. The `num_model_requests` field counts requests, not individual questions: a request with several questions counts once. See the [Decisions usage reference](https://developers.openai.com/api/reference/resources/admin/subresources/organization/subresources/usage/methods/decisions) for filters and pagination.

## Pricing and availability

With `gpt-6-luna`, uncached input costs **$0.10 per 1M tokens**. Prompt cache hits receive discounted input pricing; cache writes are charged. Cache-read and cache-write rates match the [standard `gpt-6-luna` prompt caching rates](https://developers.openai.com/api/docs/pricing). Output tokens are not charged.

Regional processing premiums and long-context input pricing multipliers apply. These rates apply to `/v1/decisions`; other requests using `gpt-6-luna` follow the applicable [model and processing-tier pricing](https://developers.openai.com/api/docs/pricing).

The Decisions API supports Zero Data Retention (ZDR) and HIPAA use for eligible customers. Data residency is available in all supported API regions. Regional processing is supported in the United States and Europe (EEA + Switzerland). See [data controls](https://developers.openai.com/api/docs/guides/your-data) for eligibility requirements, required agreements, and limitations.

## Add voice control

[Connect voice to Decisions](https://developers.openai.com/api/docs/guides/decisions-voice) to choose an action from a spoken request or react as your user or GPT-Live speaks.