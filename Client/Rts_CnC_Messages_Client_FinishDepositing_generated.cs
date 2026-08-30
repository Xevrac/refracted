using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_FinishDepositing
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.FinishDepositing); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.FinishDepositing)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize CollectorId
            s.Write(value.CollectorId);
            //  Serialize ResourceType
            s.Write(value.ResourceType);
            //  Serialize HarvesterAmount
            s.Write(value.HarvesterAmount);
            //  Serialize DisengageTimeMS
            s.Write(value.DisengageTimeMS);
            //  Serialize AmountDeposited
            s.Write(value.AmountDeposited);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.FinishDepositing)) as Rts.CnC.Messages.Client.FinishDepositing;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize CollectorId
            s.Read(out value.CollectorId);
            //  Deserialize ResourceType
            s.Read(out value.ResourceType);
            //  Deserialize HarvesterAmount
            s.Read(out value.HarvesterAmount);
            //  Deserialize DisengageTimeMS
            s.Read(out value.DisengageTimeMS);
            //  Deserialize AmountDeposited
            s.Read(out value.AmountDeposited);

            return value;
        }
        
    }
}
