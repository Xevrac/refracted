using System;
using Rts.Serialization.StreamExtensions;

namespace GeneratedSerializers.CurrentVersion.Rts_CnC_Messages_Client_RequestDeposit
{
    public class Serializer
    {
        public static Type SerializedType{ get { return typeof(Rts.CnC.Messages.Client.RequestDeposit); } }
        
        public static void Serialize(System.IO.Stream s, object obj)
        {
            var value = (Rts.CnC.Messages.Client.RequestDeposit)obj;
            //  Serialize PlayerId
            s.Write(value.PlayerId);
            //  Serialize HarvesterId
            s.Write(value.HarvesterId);
            //  Serialize CollectorId
            s.Write(value.CollectorId);
            //  Serialize ResourceType
            s.Write(value.ResourceType);

        }
        
        public static object Deserialize(System.IO.Stream s)
        {
            var value = System.Runtime.Serialization.FormatterServices.GetUninitializedObject(typeof(Rts.CnC.Messages.Client.RequestDeposit)) as Rts.CnC.Messages.Client.RequestDeposit;
            //  Deserialize PlayerId
            s.Read(out value.PlayerId);
            //  Deserialize HarvesterId
            s.Read(out value.HarvesterId);
            //  Deserialize CollectorId
            s.Read(out value.CollectorId);
            //  Deserialize ResourceType
            s.Read(out value.ResourceType);

            return value;
        }
        
    }
}
