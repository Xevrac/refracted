using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_InitializeCustomPlayerPower
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.InitializeCustomPlayerPower); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.InitializeCustomPlayerPower)obj;
            //  Serialize Id
            s.Write(value.Id);
            //  Serialize PackedData
            s.Write(value.PackedData);
            //  Serialize IconImage
            s.Write(value.IconImage);
            //  Serialize HalNameId
            s.Write(value.HalNameId);
            //  Serialize HalDescriptionId
            s.Write(value.HalDescriptionId);
            //  Serialize HalGameplayNotesId
            s.Write(value.HalGameplayNotesId);
            //  Serialize RechargeTime
            s.Write(value.RechargeTime);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.InitializeCustomPlayerPower)) as Rts.CnC.Messages.Client.InitializeCustomPlayerPower;
            //  Deserialize Id
            s.Read(out value.Id);
            //  Deserialize PackedData
            s.Read(out value.PackedData);
            //  Deserialize IconImage
            s.Read(out value.IconImage);
            //  Deserialize HalNameId
            s.Read(out value.HalNameId);
            //  Deserialize HalDescriptionId
            s.Read(out value.HalDescriptionId);
            //  Deserialize HalGameplayNotesId
            s.Read(out value.HalGameplayNotesId);
            //  Deserialize RechargeTime
            s.Read(out value.RechargeTime);

            return value;
        }
        
    }
}
