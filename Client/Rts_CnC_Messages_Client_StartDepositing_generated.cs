using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_StartDepositing
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.StartDepositing); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.StartDepositing)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize CollectorId
            s.Write(value.CollectorId);
            //  Serialize ResourceType
            s.Write(value.ResourceType);
            //  Serialize AmountToDeposit
            s.Write(value.AmountToDeposit);
            //  Serialize EngageTimeMS
            s.Write(value.EngageTimeMS);
            //  Serialize DepositTimeMS
            s.Write(value.DepositTimeMS);
            //  Serialize DisengageTimeMS
            s.Write(value.DisengageTimeMS);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.StartDepositing)) as Rts.CnC.Messages.Client.StartDepositing;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize CollectorId
            s.Read(out value.CollectorId);
            //  Deserialize ResourceType
            s.Read(out value.ResourceType);
            //  Deserialize AmountToDeposit
            s.Read(out value.AmountToDeposit);
            //  Deserialize EngageTimeMS
            s.Read(out value.EngageTimeMS);
            //  Deserialize DepositTimeMS
            s.Read(out value.DepositTimeMS);
            //  Deserialize DisengageTimeMS
            s.Read(out value.DisengageTimeMS);

            return value;
        }
        
    }
}
