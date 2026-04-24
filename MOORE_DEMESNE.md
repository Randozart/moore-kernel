# The Moore Demesne

## Feudal Computing: The Chancery of Silicon

The Moore Kernel reimagines FPGA resource management through the lens of medieval property law. The fabric is not merely "memory" or "logic cells"—it is **land**, subject to tenure, survey, and the ancient rites of seisin.

Empty space is not void—it is the **Demesne**, the totality of the FPGA, waiting to be parceled into Plots. A `.writ` tetheres a program to a Plot, making the program **Appurtenant** to that coordinate of silicon.

---

## Core Vocabulary

### The Demesne
*The Lord's Land*

In feudal law, the *demesne* was the land retained by the Lord for his own use, not leased to tenants.

In Moore, the **Demesne** is the total totality of the FPGA's logic cells, block RAM, and DSP slices—the "Great Field" from which Plots are surveyed.

> *"The Demesne is the Lord's; the Plot is the Servant's."*

### The Plot
*The Surveyed Coordinate*

When the kernel divides the FPGA fabric for different programs, it is "**parceling out the Demesne**." Each subdivision is a **Plot**.

Instead of a memory address, a program has a **"Situs"**—the legal location of a property within the Demesne.

### Seised (Verb: to Seise)
*The Taking of Possession*

*Seisin* is the ancient legal term for possession of land. If you are "Seised of a Plot," you legally own it. It sounds like "Seized," implying the program has firmly taken control of the hardware.

> `"The .writ has been served; the Plot is now Seised."`

When the kernel loads a program onto a Plot, the program is **Seised** of that silicon.

**Dutch alternative:** **"Bezet"** (Occupied/Taken) or **"Beleend"** (Enfeoffed/Loaned).

### Appurtenant
*The Program That Belongs*

In law, something "**appurtenant**" is a right that belongs to a piece of land—it cannot exist independently of that land.

An **Appurtenant** is the program that is legally tethered to a Plot. It does not merely "run" there; it **belongs to** that specific coordinate of silicon. It is an extension of the hardware's own being.

> *"The Appurtenant is the soul of the Plot; it knows the Stone, for it is of the Stone. It is not an App that wanders, but a Right that Dwells."*

**Linguistic note:** "Appurtenant" is the etymological root of the modern word "App" (Application). In the Chancery of Moore, we do not have "Apps"—we have **Appurtenants**.

### The Tenure
*The Right to Dwell*

A program's runtime is its **Tenure**—the period during which it is Seised of the Plot.

### Escheat
*The Reclamation*

If an Appurtenant crashes, is killed, or terminates without heir, the Plot **Escheats** to the Demesne.

*Escheat:* In feudal law, when a tenant died without an heir, the property returned to the Lord.

> `[WARN] Appurtenant 0x4F has crashed. Plot 02 escheated to the Demesne.`

---

## The Appurtenant Litany

> *"The Appurtenant is the soul of the Plot; it knows the Stone, for it is of the Stone. It is not an App that wanders, but a Right that Dwells."*

---

## The Thomas More Property Link

Thomas More, as a lawyer, would have dealt with "Appurtenances" every day—rights of way, grazing rights, and other incorporeal rights that belong to land.

By calling programs **Appurtenants**, we assert that code is not a "product" you buy; it is a **Right** you exercise over the silicon you own.

This turns the FPGA into a **Community of Rights** rather than a "Platform of Services."

---

## The "App-artment" Pun

Since the Moore Kernel is conceived in the Netherlands:

*   **Appartement** (Apartment) adds another layer of meaning.
*   A **Plot** is like an apartment in the **Demesne**.
*   The **Appurtenant** is the tenant living in the **Appartement**.
*   The **.writ** is the lease.

---

## Moore CLI Examples

```bash
> moore survey
Demesne: 40k Logic Cells
Plots:
  Plot 01: [VACANT]
  Plot 02: [SEISED] by kernel.writ (Tenure: 14:02)

> serve app.writ --to plot 03
Counsel: Serving Writ...
Moore: Plot 03 is now Seised.
Moore: Logic is Appurtenant to the Situs.
```

---

## Glossary for the Codex Cancellarii

| Term | Definition |
|------|------------|
| **The Demesne** | The totality of the Silicon Fabric |
| **The Plot** | A surveyed coordinate of logic gates |
| **The Appurtenant** | The program (formerly "App") legally tethered to a Plot |
| **The Seisin** | The state of an Appurtenant being "live" on the hardware |
| **The Escheat** | The reclamation of the Plot when the Appurtenant fails |
| **The Tenure** | The runtime of an Appurtenant on a Plot |
| **The Situs** | The legal address/location of a Plot within the Demesne |
| **The Appurtenance** | The Appurtenant's right to exist on a Plot |

---

## IN LOGICA GRAVITAS, IN SILICA VERITAS

*Counsel rests. The Appurtenancy is in session.*

**"Your PC runs on Moore, but keeps it Brief."**
*(The Appurtenants are seised. The Demesne is secure.)*
