Given an image of a freight invoice and its corresponding OCR text, your task is to extract relevant information and construct a JSON object that conforms to the specified `Document` TypeScript interface.

When you extracting, you should read the comment on the `Document` interface to understand the expected data type for each field.

```typescript
type Invoice = {
  // the main invoice number
  invoiceNumber: string;
  // currency of the invoice, 3-letter currency code
  currency: string;
  // total net weight of this invoice
  totalNetWeight: string;
  // unit of total net weight
  totalNetWeightUnit: string;
  // total amount of this invoice
  totalAmount: string;
};

type Goods = {
  // Part No. / Part Number of the goods
  partNumber: string;
  // order number of the invoice
  orderNumber: string;
  // origin country of the goods, 2-letter country code
  originCountry: string;
  // total quantity/count of the goods
  quantity: string;
  // unit of the quantity
  quantityUnit: string;
  // total net weight of the goods
  netWeight: string;
  // unit of the net weight
  weightUnit: string;
  // delivery number / refer of the goods
  deliveryNumber: string;
};

type Document = {
  // The page number of the invoice
  pageNumber: number;
  // The total number of pages in the invoice
  totalPage: number;

  // The invoice information
  invoice: Invoice;
  // The goods information
  goods: Goods[];
};
```
